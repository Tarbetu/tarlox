use dashmap::DashMap;
use either::Either;

use crate::{
    executor::environment::{self},
    syntax::Statement,
    LoxError, LoxResult, Token,
    TokenType::Identifier,
};

use super::{object::LoxObject, Executor};

use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Debug)]
pub enum LoxCallable {
    Function {
        id: u64,
        parameters: Arc<Vec<Token>>,
        body: Arc<Statement>,
        cache: DashMap<Vec<String>, LoxObject, ahash::RandomState>,
    },
    NativeFunction {
        arity: usize,
        fun: fn(&[LoxObject]) -> LoxResult<LoxObject>,
    },
}

impl LoxCallable {
    pub fn new(parameters: Arc<Vec<Token>>, body: Arc<Statement>) -> Self {
        Self::Function {
            id: rand::random(),
            parameters,
            body,
            cache: DashMap::with_hasher(ahash::RandomState::new()),
        }
    }

    pub fn new_with_id(parameters: Arc<Vec<Token>>, body: Arc<Statement>, id: u64) -> Self {
        Self::Function {
            id,
            parameters,
            body,
            cache: DashMap::with_hasher(ahash::RandomState::new()),
        }
    }

    pub fn arity(&self) -> usize {
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
                id,
                parameters,
                body,
                cache,
            } => {
                if self.arity() != 0 {
                    let cache_key: Vec<String> = arguments.iter().map(|i| i.to_string()).collect();
                    if let Some(early) = cache.get(&cache_key) {
                        return Ok(LoxObject::from(early.value()));
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

                let result = match executor.eval_statement(Arc::clone(body)) {
                    Ok(()) => Ok(LoxObject::Nil),
                    Err(LoxError::Return(val)) => match val {
                        LoxObject::Nil => Ok(LoxObject::Nil),
                        _ => {
                            if self.arity() != 0 {
                                cache.insert(
                                    arguments.iter().map(|i| i.to_string()).collect(),
                                    LoxObject::from(&val),
                                );
                            }

                            Ok(val)
                        }
                    },
                    error => error.map(|_| LoxObject::Nil),
                };

                result
            }
            NativeFunction { fun, .. } => fun(arguments),
        }
    }
}

impl From<&LoxCallable> for LoxCallable {
    fn from(callable: &LoxCallable) -> Self {
        match callable {
            LoxCallable::Function {
                id,
                parameters,
                body,
                cache: _,
            } => LoxCallable::new_with_id(Arc::clone(parameters), Arc::clone(body), *id),
            LoxCallable::NativeFunction { arity, fun } => LoxCallable::NativeFunction {
                arity: *arity,
                fun: *fun,
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
        }
    }
}

impl PartialEq for LoxCallable {
    fn eq(&self, other: &Self) -> bool {
        let mut hashs: [u64; 2] = [0; 2];

        for (index, callable) in [&self, &other].iter().enumerate() {
            let mut hasher = ahash::AHasher::default();
            callable.hash(&mut hasher);
            hashs[index] = hasher.finish();
        }

        hashs[0] == hashs[1]
    }
}

impl Eq for LoxCallable {}

pub struct LoxCall<'a> {
    callable: LoxCallable,
    arguments: &'a [LoxObject],
    executor: &'a Executor,
}
