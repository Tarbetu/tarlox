use dashmap::DashMap;
use either::Either;

use crate::{
    executor::environment::{self, env_hash},
    syntax::Statement,
    LoxError, LoxResult, Token,
    TokenType::Identifier,
};

use super::{object::LoxObject, Executor};

use std::sync::Arc;

pub enum LoxCallable {
    Function {
        parameters: Arc<Vec<Token>>,
        body: Arc<Statement>,
        cache: DashMap<Vec<String>, Either<LoxObject, LoxCallable>, ahash::RandomState>,
    },
    NativeFunction {
        arity: usize,
        fun: fn(&[LoxObject]) -> LoxResult<LoxObject>,
    },
}

impl LoxCallable {
    // Add arity check
    pub fn new(parameters: Arc<Vec<Token>>, body: Arc<Statement>) -> Self {
        Self::Function {
            parameters,
            body,
            cache: DashMap::with_hasher(ahash::RandomState::new()),
        }
    }

    fn arity(&self) -> usize {
        use LoxCallable::*;

        match self {
            Function { parameters, .. } => parameters.len(),
            NativeFunction { arity, .. } => *arity,
        }
    }

    pub fn call(
        &self,
        executor: &Executor,
        arguments: &[LoxObject],
    ) -> LoxResult<Either<LoxObject, LoxCallable>> {
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
            } => {
                if self.arity() != 0 {
                    let cache_key: Vec<String> = arguments.iter().map(|i| i.to_string()).collect();
                    if let Some(early) = cache.get(&cache_key) {
                        return Ok(Either::Left(LoxObject::from(
                            early.value().as_ref().left().unwrap(),
                        )));
                    };
                }

                for (index, param) in parameters.iter().enumerate() {
                    if let Identifier(name) = &param.kind {
                        environment::put_immediately(
                            Arc::clone(&executor.environment),
                            name,
                            Either::Right(arguments.get(index).unwrap().into()),
                        )
                    }
                }

                let evaluated_statement =
                    stacker::maybe_grow(1024 * 1024, 100 * 1024 * 1024, || {
                        executor.eval_statement(Arc::clone(body))
                    });

                let result = match evaluated_statement {
                    Ok(()) => Ok(Either::Left(LoxObject::Nil)),
                    Err(LoxError::Return) => {
                        match executor
                            .environment
                            .remove(&env_hash("@Return Value"))
                            .unwrap()
                            .1
                            .wait_for_value()
                        {
                            Ok(val) => match val {
                                LoxObject::Nil => Ok(Either::Left(LoxObject::Nil)),
                                LoxObject::FunctionId(id) => {
                                    let fun = executor.environment.get_function(id);
                                    if let Some(callable) = fun {
                                        Ok(Either::Right(LoxCallable::from(callable.value())))
                                    } else {
                                        Err(LoxError::InternalError(
                                            "Can't find the function while returning!".to_string(),
                                        ))
                                    }
                                }
                                _ => {
                                    if self.arity() != 0 {
                                        cache.insert(
                                            arguments.iter().map(|i| i.to_string()).collect(),
                                            Either::Left(val.into()),
                                        );
                                    }

                                    Ok(Either::Left(val.into()))
                                }
                            },
                            Err(e) => Err(e.to_owned()),
                        }
                    }
                    error => error.map(|_| Either::Left(LoxObject::Nil)),
                };

                result
            }
            NativeFunction { fun, .. } => fun(arguments).map(Either::Left),
        }
    }
}

impl From<&LoxCallable> for LoxCallable {
    fn from(callable: &LoxCallable) -> Self {
        match callable {
            LoxCallable::Function {
                parameters,
                body,
                cache: _,
            } => LoxCallable::new(Arc::clone(parameters), Arc::clone(body)),
            LoxCallable::NativeFunction { arity, fun } => LoxCallable::NativeFunction {
                arity: *arity,
                fun: *fun,
            },
        }
    }
}
