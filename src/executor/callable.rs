use either::Either;

use crate::{executor::environment, syntax::Statement, LoxError, LoxResult, Token};

use super::{object::LoxObject, Environment, Executor};

use std::sync::Arc;

pub enum LoxCallable {
    Function {
        parameters: Vec<Token>,
        body: Arc<Statement>,
        executor: Executor,
    },
    NativeFunction {
        arity: usize,
        fun: fn(&[LoxObject]) -> LoxResult<LoxObject>,
    },
}

impl LoxCallable {
    pub fn new(parameters: Vec<Token>, body: Arc<Statement>, parent_executor: &Executor) -> Self {
        Self::Function {
            parameters,
            body,
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
            } => {
                for (index, arg) in parameters.iter().enumerate() {
                    environment::put_immediately(
                        Arc::clone(&executor.environment),
                        &arg.to_string(),
                        Either::Right(arguments.get(index).unwrap().into()),
                    )
                }

                executor.eval_statement(Arc::clone(body))?;
                // Remove the result value from environment before cleaning exit
                executor.environment.clear();
                Ok(LoxObject::from(()))
            }
            NativeFunction { fun, .. } => fun(arguments),
        }
    }
}
