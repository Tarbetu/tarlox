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
        parameters: Vec<Token>,
        body: Arc<Statement>,
        cache: DashMap<Vec<String>, LoxObject, ahash::RandomState>,
    },
    NativeFunction {
        arity: usize,
        fun: fn(&[LoxObject]) -> LoxResult<LoxObject>,
    },
}

impl LoxCallable {
    // Add arity check
    pub fn new(parameters: Vec<Token>, body: Arc<Statement>) -> Self {
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

    pub fn call(&self, executor: &Executor, arguments: &[LoxObject]) -> LoxResult<LoxObject> {
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
                let cache_key: Vec<String> = arguments.iter().map(|i| i.to_string()).collect();
                if let Some(early) = cache.get(&cache_key) {
                    return Ok(early.value().into());
                };

                for (index, param) in parameters.iter().enumerate() {
                    if let Identifier(name) = &param.kind {
                        environment::put_immediately(
                            Arc::clone(&executor.environment),
                            name,
                            Either::Right(arguments.get(index).unwrap().into()),
                        )
                    }
                }

                let evaluated_statement = stacker::maybe_grow(10240 * 1024, 10240 * 1024, || {
                    executor.eval_statement(Arc::clone(body))
                });

                let result = match evaluated_statement {
                    Ok(()) => Ok(LoxObject::Nil),
                    Err(LoxError::Return) => {
                        match executor
                            .environment
                            .remove(&env_hash("@Return Value"))
                            .unwrap()
                            .1
                            .wait_for_value()
                        {
                            Ok(val) => {
                                if bool::from(&val.is_not_equal(&LoxObject::Nil)) {
                                    cache.insert(
                                        arguments.iter().map(|i| i.to_string()).collect(),
                                        val.into(),
                                    );
                                }
                                Ok(val.into())
                            }
                            Err(e) => Err(e.to_owned()),
                        }
                    }
                    error => error.map(|_| LoxObject::Nil),
                };

                result
            }
            NativeFunction { fun, .. } => fun(arguments),
        }
    }
}
