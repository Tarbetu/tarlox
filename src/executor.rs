pub mod callable;
pub mod environment;
pub mod object;

use either::Either;
use threadpool::ThreadPool;

pub use crate::executor::callable::LoxCallable;
use crate::WORKERS;
pub use object::LoxObject;

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

use std::sync::Arc;

pub struct Executor {
    environment: Arc<Environment>,
    workers: &'static ThreadPool,
}

impl Executor {
    pub fn new(workers: &'static ThreadPool, environment: Arc<Environment>) -> Executor {
        Self {
            environment,
            workers,
        }
    }

    pub fn execute(&self, statements: Arc<Vec<Arc<Statement>>>) -> LoxResult<()> {
        for statement in statements.iter() {
            self.eval_statement(Arc::clone(statement))?;
        }

        Ok(())
    }

    fn eval_statement(&self, stmt: Arc<Statement>) -> LoxResult<()> {
        use Statement::*;

        match stmt.as_ref() {
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
                        self.workers,
                        match &token.kind {
                            TokenType::Identifier(name) => name,
                            _ => unreachable!(),
                        },
                        Arc::clone(expr),
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
                let sub_executor = Executor {
                    workers: self.workers,
                    environment: Arc::new(Environment::new_with_parent(Arc::clone(&previous))),
                };

                sub_executor.execute(Arc::clone(statements))
            }
            If(condition, then_branch, else_branch) => {
                let condition = bool::from(&eval_expression(self.environment.clone(), condition)?);

                if condition {
                    self.eval_statement(Arc::clone(then_branch))?;
                } else if else_branch.is_some() {
                    self.eval_statement(Arc::clone(else_branch.as_ref().unwrap()))?;
                }

                Ok(())
            }
            While(condition, body) => {
                while bool::from(&eval_expression(Arc::clone(&self.environment), condition)?) {
                    self.eval_statement(Arc::clone(body))?;
                }

                Ok(())
            }
            Function(name, params, body) => {
                if let TokenType::Identifier(name) = &name.kind {
                    let fun = LoxCallable::new(Arc::new(params.to_owned()), Arc::clone(body));
                    let fun_hash = environment::env_hash(name);
                    environment::put_function(Arc::clone(&self.environment), fun_hash, fun);
                    Ok(())
                } else {
                    Err(LoxError::ParseError {
                        line: Some(name.line),
                        msg: String::from("Invalid name specified in function statement!"),
                    })
                }
            }
            Return(maybe_expr) => {
                environment::put_immediately(
                    if let Some(enclosing) = &self.environment.enclosing {
                        Arc::clone(enclosing)
                    } else {
                        Arc::clone(&self.environment)
                    },
                    "@Return Value",
                    if let Some(expr) = maybe_expr {
                        Either::Left(expr)
                    } else {
                        Either::Right(LoxObject::Nil)
                    },
                );

                Err(LoxError::Return)
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
                    let name = environment::env_hash(name);
                    if let Some(var) = environment.get(&name) {
                        match var.value() {
                            PackagedObject::Ready(_) => Ok(LoxObject::from(true)),
                            PackagedObject::Pending(..) => Ok(LoxObject::from(false)),
                        }
                    } else {
                        Err(LoxError::RuntimeError {
                            line: Some(tkn.line),
                            msg: "Undefined variable while is_ready call".into(),
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
                let result = environment.get(&environment::env_hash(name));

                if let Some(pair) = result {
                    match pair.value().wait_for_value() {
                        Ok(obj) => Ok(LoxObject::from(obj)),
                        Err(e) => Err(e.clone()),
                    }
                } else {
                    Err(LoxError::RuntimeError {
                        line: Some(token.line),
                        msg: format!("Undefined variable '{name}'"),
                    })
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
                let hash = environment::env_hash(name);
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
                        msg: "Undefined variable while assign".into(),
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
        Call(callee, paren, arguments) => {
            let callee_id = eval_expression(Arc::clone(&environment), callee)?;

            if let LoxObject::FunctionId(hash) = callee_id {
                let arguments = {
                    let mut res = vec![];

                    for arg in arguments {
                        res.push(eval_expression(Arc::clone(&environment), arg)?)
                    }

                    res
                };

                match environment.get(&hash) {
                    Some(pair) => match pair.value().wait_for_value() {
                        Ok(LoxObject::FunctionId(fun_hash)) => {
                            if let Some(fun) = environment.get_function(fun_hash) {
                                let sub_executor = Executor {
                                    workers: &WORKERS,
                                    environment: Arc::new(Environment::new_with_parent(
                                        Arc::clone(&environment),
                                    )),
                                };

                                match fun.call(&sub_executor, &arguments) {
                                    Ok(Either::Left(obj)) => Ok(obj),
                                    Ok(Either::Right(_fun)) => unimplemented!(),
                                    Err(e) => Err(e),
                                }
                            } else {
                                Err(LoxError::InternalError(String::from(
                                    "FunctionId does not point a function!",
                                )))
                            }
                        }
                        Ok(obj) => Err(LoxError::InternalError(format!(
                            "Callee does not reference to an function! It references to an {}",
                            obj.to_string()
                        ))),
                        Err(_) => Err(LoxError::InternalError(String::from(
                            "Invalid state for callee!",
                        ))),
                    },
                    None => Err(LoxError::RuntimeError {
                        line: Some(paren.line),
                        msg: format!("Undefined callable in func '{}'", hash),
                    }),
                }
            } else {
                Err(LoxError::InternalError("Can't find function hash!".into()))
            }
        }
    }
}
