mod environment;
mod object;

use async_recursion::async_recursion;
use either::Either;
use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::executor::environment::PackagedObject;
use crate::syntax::expression::LoxLiteral;
use crate::syntax::expression::Operator;
use crate::syntax::Expression;
use crate::syntax::Statement;
use crate::LoxError;
use crate::LoxResult;
use crate::TokenType;
use crate::TokenType::*;
pub use environment::Environment;
use object::LoxObject;

use std::sync::Arc;
use std::{num::NonZeroUsize, thread::available_parallelism};

pub struct Executor {
    environment: Arc<Environment>,
    workers: ThreadPool,
}

impl Executor {
    pub fn new() -> Executor {
        Self {
            environment: Arc::new(Environment::new()),
            workers: ThreadPoolBuilder::new()
                .num_threads(
                    available_parallelism()
                        .unwrap_or(NonZeroUsize::new(1).unwrap())
                        .into(),
                )
                .build()
                .unwrap(),
        }
    }

    #[async_recursion]
    pub async fn execute(&mut self, statements: &[Statement]) -> LoxResult<()> {
        for statement in statements {
            self.eval_statement(statement).await?;
        }

        Ok(())
    }

    #[async_recursion]
    async fn eval_statement(&mut self, stmt: &Statement) -> LoxResult<()> {
        use Statement::*;

        match stmt {
            StmtExpression(expr) => {
                eval_expression(Arc::clone(&self.environment), expr)?;

                Ok(())
            }
            Print(expr) => {
                let res = eval_expression(Arc::clone(&self.environment), expr)?;

                println!("{}", res.to_string());
                Ok(())
            }
            Var(token, initializer) => {
                if let Some(expr) = initializer {
                    environment::put(
                        Arc::clone(&self.environment),
                        &self.workers,
                        match &token.kind {
                            TokenType::Identifier(name) => name,
                            _ => unreachable!(),
                        },
                        expr,
                    );

                    Ok(())
                } else {
                    environment::put_immediately(
                        Arc::clone(&self.environment),
                        match &token.kind {
                            TokenType::Identifier(name) => name,
                            _ => unreachable!(),
                        },
                        Either::Right(LoxObject::Nil),
                    );

                    Ok(())
                }
            }
            AwaitVar(token, initializer) => {
                environment::put_immediately(
                    Arc::clone(&self.environment),
                    match &token.kind {
                        TokenType::Identifier(name) => name,
                        _ => unreachable!(),
                    },
                    Either::Left(initializer),
                );

                Ok(())
            }
            Block(statements) => {
                let previous = Arc::clone(&self.environment);
                self.environment = Arc::new(Environment::new_with_parent(Arc::clone(&previous)));

                let res = self.execute(statements).await;

                self.environment = previous;

                res
            }
            If(condition, then_branch, else_branch) => {
                let condition = bool::from(&eval_expression(self.environment.clone(), condition)?);

                if condition {
                    self.eval_statement(then_branch).await?;
                } else if else_branch.is_some() {
                    self.eval_statement(else_branch.as_ref().unwrap()).await?;
                }

                Ok(())
            }
            While(condition, body) => {
                while bool::from(&eval_expression(Arc::clone(&self.environment), condition)?) {
                    self.eval_statement(body).await?;
                }

                return Ok(());
            }
        }
    }
}

fn eval_expression(environment: Arc<Environment>, expr: &Expression) -> LoxResult<LoxObject> {
    use Expression::*;
    use LoxLiteral::*;

    match expr {
        Grouping(inner) => eval_expression(environment, inner),
        Literal(Number(n)) => Ok(LoxObject::from(n)),
        Literal(LoxString(s)) => Ok(LoxObject::from(s.as_str())),
        Literal(Bool(b)) => Ok(LoxObject::from(*b)),
        Literal(Nil) => Ok(LoxObject::Nil),
        Unary(Operator::IsReady, right) => {
            if let Variable(tkn) = right.as_ref() {
                if let TokenType::Identifier(name) = &tkn.kind {
                    let name = environment::variable_hash(name);
                    if let Some(var) = environment.get(&name) {
                        match var.value() {
                            PackagedObject::Ready(_) => Ok(LoxObject::from(true)),
                            PackagedObject::Pending(..) => Ok(LoxObject::from(false)),
                        }
                    } else {
                        Err(LoxError::RuntimeError {
                            line: Some(tkn.line),
                            msg: "Undefined Variable".into(),
                        })
                    }
                } else {
                    unreachable!()
                }
            } else {
                // R-values are always ready
                Ok(LoxObject::from(true))
            }
        }
        Unary(operator, right) => {
            let right = eval_expression(environment, right)?;

            match operator {
                Operator::Minus => Ok(LoxObject::from(right.apply_negative()?)),
                Operator::Not => Ok(LoxObject::from(!bool::from(&right))),
                _ => unreachable!(),
            }
        }
        Binary(left, operator, right) => {
            let left = eval_expression(environment.clone(), left)?;
            let right = eval_expression(environment.clone(), right)?;

            match operator {
                Operator::Star => left * right,
                Operator::Slash => left / right,
                Operator::Minus => left - right,
                Operator::Plus => left + right,
                Operator::Equality => Ok(left.is_equal(&right)),
                Operator::NotEqual => Ok(left.is_not_equal(&right)),
                Operator::Greater => left.is_greater(&right),
                Operator::GreaterOrEqual => left.is_greater_equal(&right),
                Operator::Smaller => left.is_less(&right),
                Operator::SmallerOrEqual => left.is_less_equal(&right),
                _ => unreachable!(),
            }
        }
        Variable(token) => {
            if let Identifier(name) = &token.kind {
                loop {
                    let result = environment.get(&environment::variable_hash(name));
                    if let Some(packaged_obj) = result {
                        match packaged_obj.value() {
                            PackagedObject::Pending(mtx, cvar) => {
                                let res = mtx.lock().unwrap();

                                let _ = cvar.wait_while(res, |pending| !*pending);
                            }
                            PackagedObject::Ready(val) => match val {
                                Ok(obj) => return Ok(LoxObject::from(obj)),
                                // Make this better in future
                                Err(e) => return Err(e.clone()),
                            },
                        }
                    } else {
                        return Err(LoxError::RuntimeError {
                            line: Some(token.line),
                            msg: "Undefined Variable".into(),
                        });
                    }
                }
            } else {
                Err(LoxError::InternalError(format!(
                    "Unexcepted Token! Excepted Identifier found {:?}",
                    token.kind
                )))
            }
        }
        Assign(name_tkn, value_expr) => {
            if let Identifier(name) = &name_tkn.kind {
                let hash = environment::variable_hash(name);
                if let Some((key, old_val)) = environment.remove(&hash) {
                    let sub_env = Arc::new(Environment::new_with_parent(Arc::clone(&environment)));
                    sub_env.values.insert(key, old_val);
                    let val = eval_expression(Arc::clone(&sub_env), value_expr)?;
                    if let Some(parent_env) = &environment.enclosing {
                        environment::put_immediately(
                            Arc::clone(parent_env),
                            name,
                            Either::Right(LoxObject::from(&val)),
                        );
                    } else {
                        environment::put_immediately(
                            Arc::clone(&environment),
                            name,
                            Either::Right(LoxObject::from(&val)),
                        );
                    }

                    Ok(val)
                } else {
                    Err(LoxError::RuntimeError {
                        line: Some(name_tkn.line),
                        msg: "Undefined Variable".into(),
                    })
                }
            } else {
                Err(LoxError::InternalError(format!(
                    "Unexcepted Token! Excepted Identifier found {:?}",
                    name_tkn.kind
                )))
            }
        }
        Logical(right, operator, left) => {
            let left = eval_expression(Arc::clone(&environment), left)?;

            if let Operator::Or = operator {
                if bool::from(&left) {
                    return Ok(left);
                }
            } else if !bool::from(&left) {
                return Ok(left);
            }

            eval_expression(environment, right)
        }
    }
}
