mod environment;
mod object;

use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::syntax::expression::LoxLiteral;
use crate::syntax::expression::Operator;
use crate::syntax::Expression;
use crate::syntax::Statement;
use crate::LoxResult;
use crate::TokenType;
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
                        either::Either::Right(LoxObject::Nil),
                    );

                    Ok(())
                }
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
            use crate::LoxError;
            use crate::TokenType::*;
            use environment::PackagedObject;

            if let Identifier(name) = &token.kind {
                loop {
                    let result = environment.values.get(&environment::variable_hash(name));
                    if let Some(packaged_obj) = result {
                        match packaged_obj.value() {
                            PackagedObject::Pending(mtx, cvar) => {
                                let mut res = mtx.lock().unwrap();

                                while !*res {
                                    res = cvar.wait(res).unwrap();
                                }
                            }
                            PackagedObject::Ready(val) => match val {
                                Ok(obj) => return Ok((obj).into()),
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
    }
}
