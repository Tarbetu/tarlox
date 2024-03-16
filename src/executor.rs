pub mod callable;
pub mod class;
pub mod environment;
pub mod object;

use ahash::AHashMap;
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use either::Either::{self, Left, Right};
use threadpool::ThreadPool;

pub use crate::executor::callable::LoxCallable;
use crate::executor::class::LoxClass;
use crate::Token;
use crate::GLOBALS;
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

type LocalsMap = Arc<DashMap<String, usize, ahash::RandomState>>;

#[derive(Debug, Clone)]
pub struct Executor {
    environment: Arc<Environment>,
    workers: &'static ThreadPool,
    locals: LocalsMap,
}

impl Executor {
    pub fn new(workers: &'static ThreadPool) -> Executor {
        Self {
            environment: Arc::new(Environment::default()),
            workers,
            locals: Arc::new(DashMap::with_hasher(ahash::RandomState::new())),
        }
    }

    pub fn resolve(&self, expr: &Expression, depth: usize) {
        self.locals.insert(expr.to_string(), depth);
    }

    pub fn lookup_variable(
        &self,
        key: &u64,
        expr: &Expression,
    ) -> Option<Ref<'_, u64, PackagedObject, ahash::RandomState>> {
        let value = self
            .locals
            .get(&expr.to_string())
            .map(|distance| self.environment.get_at(*distance, key).unwrap());

        if value.is_some() {
            value
        } else {
            GLOBALS.get(key)
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
                self.clone().eval_expression(expr)?;

                Ok(())
            }
            Print(expr) => {
                let res = self.clone().eval_expression(expr)?;

                println!("{}", res.to_string());
                Ok(())
            }
            Var(token, initializer) => {
                if let Some(expr) = initializer {
                    environment::put(
                        Arc::clone(&self.environment),
                        Arc::clone(&self.locals),
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
                        Arc::clone(&self.locals),
                        match &token.kind {
                            TokenType::Identifier(name) => name,
                            _ => unreachable!(),
                        },
                        Right(LoxObject::Nil),
                    );

                    Ok(())
                }
            }
            AwaitVar(token, initializer) => {
                environment::put_immediately(
                    Arc::clone(&self.environment),
                    Arc::clone(&self.locals),
                    match &token.kind {
                        TokenType::Identifier(name) => name,
                        _ => unreachable!(),
                    },
                    Left(initializer),
                );

                Ok(())
            }
            Block(statements) => {
                let previous = Arc::clone(&self.environment);
                let sub_executor = Executor {
                    workers: self.workers,
                    environment: Arc::new(Environment::new_with_parent(Arc::clone(&previous))),
                    locals: Arc::clone(&self.locals),
                };

                sub_executor.execute(Arc::clone(statements))
            }
            If(condition, then_branch, else_branch) => {
                let condition = bool::from(&self.eval_expression(condition)?);

                if condition {
                    self.eval_statement(Arc::clone(then_branch))?;
                } else if else_branch.is_some() {
                    self.eval_statement(Arc::clone(else_branch.as_ref().unwrap()))?;
                }

                Ok(())
            }
            While(condition, body) => {
                while bool::from(&self.eval_expression(condition)?) {
                    self.eval_statement(Arc::clone(body))?;
                }

                Ok(())
            }
            Function(name, params, body) => {
                if let TokenType::Identifier(name) = &name.kind {
                    let fun = LoxCallable::new(Arc::new(params.to_owned()), Arc::clone(body), true);
                    environment::put_immediately(
                        Arc::clone(&self.environment),
                        Arc::clone(&self.locals),
                        name,
                        Right(LoxObject::from(fun)),
                    );
                    Ok(())
                } else {
                    Err(LoxError::ParseError {
                        line: Some(name.line),
                        msg: String::from("Invalid name specified in function statement!"),
                    })
                }
            }
            Return(maybe_expr) => Err(LoxError::Return(
                Arc::clone(&self.environment),
                maybe_expr.as_ref().map(Arc::clone),
            )),
            Class(name, methods) => {
                if let TokenType::Identifier(name) = &name.kind {
                    let methods = {
                        let mut result = AHashMap::new();

                        for method in methods {
                            if let Statement::Function(
                                Token {
                                    line: _,
                                    kind: TokenType::Identifier(method_name),
                                },
                                params,
                                body,
                            ) = method
                            {
                                result.insert(
                                    method_name.to_owned(),
                                    // Param should be an integer
                                    LoxCallable::new(
                                        Arc::new(params.to_owned()),
                                        Arc::clone(body),
                                        false,
                                    ),
                                );
                            } else {
                                unreachable!()
                            }
                        }

                        result
                    };

                    environment::put_immediately(
                        Arc::clone(&self.environment),
                        Arc::clone(&self.locals),
                        name,
                        Right(LoxObject::from(LoxCallable::Class {
                            class: Arc::new(LoxClass::new(name.to_string(), methods)),
                        })),
                    );

                    Ok(())
                } else {
                    Err(LoxError::ParseError {
                        line: Some(name.line),
                        msg: String::from("Invalid name specified in function statement!"),
                    })
                }
            }
        }
    }

    fn eval_expression(&self, expr: &Expression) -> LoxResult<LoxObject> {
        use Expression::*;
        use LoxLiteral::*;

        match expr {
            Grouping(inner) => self.eval_expression(inner),
            Literal(Number(n)) => Ok(LoxObject::from(n)),
            Literal(LoxString(s)) => Ok(LoxObject::from(s.as_str())),
            Literal(Bool(b)) => Ok(LoxObject::from(*b)),
            Literal(Nil) => Ok(LoxObject::Nil),
            Unary(Operator::IsReady, right) => {
                if let Variable(tkn) = right.as_ref() {
                    if let TokenType::Identifier(name) = &tkn.kind {
                        let name = environment::env_hash(name);
                        if let Some(var) = self.environment.get(&name) {
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
                let right = self.eval_expression(right)?;

                match operator {
                    Operator::Minus => Ok(LoxObject::from(&right.apply_negative()?)),
                    Operator::Not => Ok(LoxObject::from(!bool::from(&right))),
                    _ => unreachable!(),
                }
            }
            Binary(left, operator, right) => {
                let left = self.clone().eval_expression(left)?;
                let right = self.clone().eval_expression(right)?;

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
                        let result = self.lookup_variable(&environment::env_hash(name), expr);

                        if let Some(pair) = result {
                            match pair.value() {
                                PackagedObject::Pending(mtx, cvar) => {
                                    let lock = mtx.lock().unwrap();

                                    let _ = cvar.wait_while(lock, |pending| !*pending);
                                }
                                PackagedObject::Ready(res) => match res {
                                    Ok(obj) => return Ok(LoxObject::from(obj)),
                                    Err(e) => return Err(e.into()),
                                },
                            }
                        } else {
                            return Err(LoxError::RuntimeError {
                                line: Some(token.line),
                                msg: format!("Undefined variable '{name}'"),
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
                    let hash = environment::env_hash(name);
                    if let Some((key, old_val)) = self.environment.remove(&hash) {
                        let sub_env =
                            Arc::new(Environment::new_with_parent(Arc::clone(&self.environment)));
                        sub_env.values.insert(key, old_val);
                        let val = self.clone().eval_expression(value_expr)?;
                        if let Some(parent_env) = &self.environment.enclosing {
                            environment::put_immediately(
                                Arc::clone(parent_env),
                                Arc::clone(&self.locals),
                                name,
                                Either::Right(LoxObject::from(&val)),
                            );
                        } else {
                            environment::put_immediately(
                                Arc::clone(&self.environment),
                                Arc::clone(&self.locals),
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
                let left = self.clone().eval_expression(left)?;

                if let Operator::Or = operator {
                    if bool::from(&left) {
                        return Ok(left);
                    }
                } else if !bool::from(&left) {
                    return Ok(left);
                }

                self.eval_expression(right)
            }
            Call(callee, paren, arguments) => {
                let callee = self.clone().eval_expression(callee)?;

                if let LoxObject::Callable(callee) = callee {
                    let arguments = {
                        let mut res = vec![];

                        for arg in arguments {
                            res.push(self.clone().eval_expression(arg)?)
                        }

                        res
                    };

                    let sub_executor = Executor {
                        workers: &WORKERS,
                        environment: Arc::new(Environment::new_with_parent(Arc::clone(
                            &self.environment,
                        ))),
                        locals: Arc::clone(&self.locals),
                    };

                    callee.call(&sub_executor, arguments)
                } else {
                    Err(LoxError::RuntimeError {
                        line: Some(paren.line),
                        msg: "Callee does not match with a function!".into(),
                    })
                }
            }
            Lambda(params, body) => Ok(LoxObject::from(LoxCallable::new(
                Arc::new(params.to_owned()),
                Arc::clone(body),
                true,
            ))),

            Get(object, name) => {
                let object = self.eval_expression(object)?;

                object.get(name)
            }
            Set(object, name, value) => {
                let object = self.eval_expression(object)?;

                if let LoxObject::Instance(..) = object {
                    let value = self.eval_expression(value)?;
                    object.set(name, value)
                } else {
                    Err(LoxError::RuntimeError {
                        line: Some(name.line),
                        msg: "Only instances have fields".into(),
                    })
                }
            }
        }
    }
}
