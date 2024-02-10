mod environment;
mod object;

use crate::syntax::expression::LoxLiteral;
use crate::syntax::expression::Operator;
use crate::syntax::Expression;
use crate::syntax::Statement;
use crate::LoxResult;
pub use environment::Environment;
use object::LoxObject;

use parking_lot::Mutex;
use std::sync::Arc;

pub struct Executor {
    global: Arc<Mutex<Environment>>,
}

impl<'a> Executor {
    pub fn new() -> Executor {
        Self {
            global: Arc::new(Mutex::new(Environment::new())),
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
                eval_expression(self.global.clone(), expr)?;

                Ok(())
            }
            Print(expr) => {
                let res = eval_expression(self.global.clone(), expr)?;

                println!("{}", res.to_string());
                Ok(())
            }
            Ready(_expr) => {
                // If expr is a identifier, check if it's accessable
                unimplemented!()
            }
            Var(token, initializer) => {
                if let Some(expr) = initializer {
                    environment::put(self.global.clone(), token.to_string(), expr);

                    Ok(())
                } else {
                    environment::put_immediately(
                        self.global.clone(),
                        token.to_string(),
                        either::Either::Right(LoxObject::Nil),
                    );

                    Ok(())
                }
            }
        }
    }
}

fn eval_expression(
    environment: Arc<Mutex<Environment>>,
    expr: &Expression,
) -> LoxResult<LoxObject> {
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
                dbg!(&environment);
                let mut locked_env = environment.lock();
                let result = locked_env.values.get_mut(name);
                if let Some(packaged_obj) = result {
                    match packaged_obj {
                        PackagedObject::Pending(rec) => {
                            let res = rec.recv()??;
                            *packaged_obj = PackagedObject::Ready(Ok((&res).into()));
                            Ok(res)
                        }
                        PackagedObject::Ready(val) => match val {
                            Ok(obj) => Ok((&*obj).into()),
                            Err(e) => Err(e.clone()),
                        },
                    }
                } else {
                    Err(LoxError::RuntimeError {
                        line: Some(token.line),
                        msg: "Undefined Variable".into(),
                    })
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
