mod environment;
mod object;

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
    global: Arc<Environment>,
    workers: ThreadPool,
}

impl<'a> Executor {
    pub fn new() -> Executor {
        Self {
            global: Arc::new(Environment::new()),
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

    pub async fn execute(&mut self, statements: Vec<Statement>) -> LoxResult<()> {
        for statement in statements {
            self.eval_statement(&statement).await?;
        }

        Ok(())
    }

    async fn eval_statement(&mut self, stmt: &Statement) -> LoxResult<()> {
        use Statement::*;

        match stmt {
            StmtExpression(expr) => {
                eval_expression(Arc::clone(&self.global), expr)?;

                Ok(())
            }
            Print(expr) => {
                let res = eval_expression(Arc::clone(&self.global), expr)?;

                println!("{}", res.to_string());
                Ok(())
            }
            Var(token, initializer) => {
                if let Some(expr) = initializer {
                    environment::put(
                        Arc::clone(&self.global),
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
                        Arc::clone(&self.global),
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
                    Arc::clone(&self.global),
                    match &token.kind {
                        TokenType::Identifier(name) => name,
                        _ => unreachable!(),
                    },
                    Either::Left(initializer),
                );

                Ok(())
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
                    if let Some(var) = environment.values.get(&name) {
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
                    let result = environment.values.get(&environment::variable_hash(name));
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
                if environment
                    .values
                    .get(&environment::variable_hash(name))
                    .is_none()
                {
                    return Err(LoxError::RuntimeError {
                        line: Some(name_tkn.line),
                        msg: "Undefined Variable".into(),
                    });
                }

                let val = eval_expression(Arc::clone(&environment), value_expr);
                environment::put_immediately(
                    environment,
                    name,
                    Either::Right(match val {
                        Ok(ref obj) => LoxObject::from(obj),
                        err => return err,
                    }),
                );
                val
            } else {
                Err(LoxError::InternalError(format!(
                    "Unexcepted Token! Excepted Identifier found {:?}",
                    name_tkn.kind
                )))
            }
        }
    }
}
