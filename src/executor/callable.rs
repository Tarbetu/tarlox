use crate::{syntax::Statement, LoxError, LoxResult, Token};

use super::{object::LoxObject, Environment, Executor};

use std::sync::Arc;

pub enum LoxCallable {
    Function {
        parameters: Vec<Token>,
        body: Box<Statement>,
        executor: Executor,
        is_recursive: bool,
    },
    NativeFunction {
        arity: usize,
        fun: fn(&[LoxObject]) -> LoxResult<LoxObject>,
    },
}

impl LoxCallable {
    pub fn new(parameters: Vec<Token>, body: Box<Statement>, parent_executor: Executor) -> Self {
        Self::Function {
            parameters,
            body,
            is_recursive: false,
            executor: Executor {
                environment: Arc::new(Environment::new_with_parent(Arc::clone(
                    &parent_executor.environment,
                ))),
                workers: parent_executor.workers,
            },
        }
    }
    fn arity(&self) -> usize {
        use LoxCallable::*;

        match self {
            Function { parameters, .. } => parameters.len(),
            NativeFunction { arity, .. } => *arity,
        }
    }

    pub fn call(&self, arguments: &[LoxObject]) -> LoxResult<LoxObject> {
        use LoxCallable::*;

        if arguments.len() > self.arity() {
            return Err(LoxError::RuntimeError {
                line: None,
                msg: "Wrong number of arguments".into(),
            });
        }

        match self {
            Function {
                parameters,
                body,
                executor,
                is_recursive,
            } => {
                if *is_recursive {
                    unimplemented!()
                } else {
                    executor.eval_statement(body)?;
                    Ok(LoxObject::Nil)
                }
            }
            NativeFunction { fun, .. } => fun(arguments),
        }
    }
}
