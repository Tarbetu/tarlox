use dashmap::DashMap;
use either::Either;
use lazy_static::lazy_static;

use crate::{
    executor::environment::{self},
    syntax::{Expression, Statement},
    LoxError, LoxResult, Token,
    TokenType::{self, Identifier},
};

use super::{class::LoxClass, object::LoxObject, Executor};

use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

lazy_static! {
    pub static ref THIS_KEY: u64 = environment::env_hash(format!("{:?}", TokenType::This).as_str());
}

#[derive(Debug)]
pub enum LoxCallable {
    Function {
        id: u64,
        parameters: Arc<Vec<Token>>,
        body: Arc<Statement>,
        cache: Option<DashMap<Vec<String>, LoxObject, ahash::RandomState>>,
        this: Option<LoxObject>,
    },
    NativeFunction {
        arity: usize,
        fun: fn(Vec<LoxObject>) -> LoxResult<LoxObject>,
    },
    Class {
        class: Arc<LoxClass>,
    },
}

impl LoxCallable {
    pub fn new(parameters: Arc<Vec<Token>>, body: Arc<Statement>) -> Self {
        Self::Function {
            id: rand::random(),
            parameters,
            body,
            cache: Some(DashMap::with_hasher(ahash::RandomState::new())),
            this: None,
        }
    }

    pub fn new_with_id(
        parameters: Arc<Vec<Token>>,
        body: Arc<Statement>,
        id: u64,
        this: Option<LoxObject>,
    ) -> Self {
        Self::Function {
            id,
            parameters,
            body,
            cache: Some(DashMap::with_hasher(ahash::RandomState::new())),
            this,
        }
    }

    pub fn new_method(parameters: Arc<Vec<Token>>, body: Arc<Statement>) -> Self {
        Self::Function {
            id: rand::random(),
            parameters,
            body,
            cache: None,
            this: None,
        }
    }

    pub fn bind(&self, this: &LoxObject) -> Self {
        if let LoxCallable::Function {
            parameters, body, ..
        } = self
        {
            LoxCallable::Function {
                id: rand::random(),
                parameters: Arc::clone(parameters),
                body: Arc::clone(body),
                cache: None,
                this: Some(LoxObject::from(this)),
            }
        } else {
            unreachable!()
        }
    }

    pub fn arity(&self) -> usize {
        use LoxCallable::*;

        match self {
            Function { parameters, .. } => parameters.len(),
            NativeFunction { arity, .. } => *arity,
            Class { .. } => 0,
        }
    }

    pub fn call(&self, executor: &Executor, mut arguments: Vec<LoxObject>) -> LoxResult<LoxObject> {
        use LoxCallable::*;

        if arguments.len() != self.arity() {
            return Err(LoxError::RuntimeError {
                line: None,
                msg: "Wrong number of arguments".into(),
            });
        }

        match self {
            Function {
                parameters,
                body,
                cache,
                this,
                ..
            } => {
                if let Some(obj) = this.as_ref() {
                    executor
                        .environment
                        .enclosing
                        .as_ref()
                        .unwrap()
                        .values
                        .insert(
                            *THIS_KEY,
                            environment::PackagedObject::Ready(Ok(LoxObject::from(obj))),
                        );
                }

                loop {
                    if let Some(cache) = cache {
                        if self.arity() != 0 {
                            let cache_key: Vec<String> =
                                arguments.iter().map(|i| i.to_string()).collect();
                            if let Some(early) = cache.get(&cache_key) {
                                return Ok(LoxObject::from(early.value()));
                            };
                        }
                    }

                    for (index, param) in parameters.iter().enumerate() {
                        if let Identifier(name) = &param.kind {
                            environment::put_immediately(
                                Arc::clone(&executor.environment),
                                Arc::clone(&executor.locals),
                                name,
                                Either::Right(arguments.get(index).unwrap().into()),
                            )
                        }
                    }

                    let result = match executor.eval_statement(Arc::clone(body)) {
                        Ok(()) => return Ok(LoxObject::Nil),
                        Err(LoxError::Return(inner_env, val)) => match val {
                            None => Ok(LoxObject::Nil),
                            Some(expr) => {
                                let sub_executor = Executor {
                                    environment: Arc::clone(&inner_env),
                                    locals: Arc::clone(&executor.locals),
                                    workers: executor.workers,
                                };
                                let val =
                                // This seems like a mess. Everywhere is filled with eval_expression!
                                if let Expression::Call(callee, _paren, uneval_inner_arguments) = expr.as_ref() {
                                    let callee = sub_executor.eval_expression(callee)?;

                                    if let LoxObject::Callable(callable) = callee {
                                        // Tail call
                                        if callable.as_ref() == self {
                                            arguments = {
                                                    let mut res = vec![];

                                                    for arg in uneval_inner_arguments {
                                                        res.push(sub_executor.eval_expression(arg)?);
                                                    }

                                                    res
                                                };

                                            continue
                                        } else {
                                            // Not a tail call
                                            sub_executor.eval_expression(&expr)?
                                        }
                                    } else {
                                        // Not callable
                                        sub_executor.eval_expression(&expr)?
                                    }
                                } else {
                                    // Not a call expression
                                    sub_executor.eval_expression(&expr)?
                                };

                                if self.arity() != 0 {
                                    cache.as_ref().and_then(|cache| {
                                        cache.insert(
                                            arguments.iter().map(|i| i.to_string()).collect(),
                                            LoxObject::from(&val),
                                        )
                                    });
                                }

                                Ok(val)
                            }
                        },
                        error => error.map(|_| LoxObject::Nil),
                    };

                    return result;
                }
            }
            NativeFunction { fun, .. } => fun(arguments),
            Class { class } => Ok(LoxObject::Instance(
                rand::random(),
                Arc::clone(class),
                Arc::new(DashMap::with_hasher(ahash::RandomState::new())),
            )),
        }
    }
}

impl From<&LoxCallable> for LoxCallable {
    fn from(callable: &LoxCallable) -> Self {
        use LoxCallable::*;
        match callable {
            Function {
                id,
                parameters,
                body,
                cache: _,
                this,
            } => LoxCallable::new_with_id(
                Arc::clone(parameters),
                Arc::clone(body),
                *id,
                this.as_ref().map(LoxObject::from),
            ),
            NativeFunction { arity, fun } => LoxCallable::NativeFunction {
                arity: *arity,
                fun: *fun,
            },
            Class { class } => Class {
                class: Arc::clone(class),
            },
        }
    }
}

impl Hash for LoxCallable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use LoxCallable::*;

        self.arity().hash(state);

        match self {
            NativeFunction { fun, .. } => fun.hash(state),
            Function { id, .. } => id.hash(state),
            Class { class } => class.name.hash(state),
        }
    }
}

impl PartialEq for LoxCallable {
    fn eq(&self, other: &Self) -> bool {
        let mut hashs: [u64; 2] = [0; 2];

        for (index, callable) in [self, other].iter().enumerate() {
            let mut hasher = ahash::AHasher::default();
            callable.hash(&mut hasher);
            hashs[index] = hasher.finish();
        }

        hashs[0] == hashs[1]
    }
}

impl Eq for LoxCallable {}
