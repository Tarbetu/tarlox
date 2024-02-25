use either::Either;

use crate::{
    executor::environment::{self},
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
    },
    NativeFunction {
        arity: usize,
        fun: fn(&[LoxObject]) -> LoxResult<LoxObject>,
    },
}

impl LoxCallable {
    // Add arity check
    pub fn new(parameters: Vec<Token>, body: Arc<Statement>) -> Self {
        Self::Function { parameters, body }
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
            Function { parameters, body } => {
                for (index, param) in parameters.iter().enumerate() {
                    if let Identifier(name) = &param.kind {
                        environment::put_immediately(
                            Arc::clone(&executor.environment),
                            name,
                            Either::Right(arguments.get(index).unwrap().into()),
                        )
                    }
                }

                executor.eval_statement(Arc::clone(body))?;
                Ok(LoxObject::from(()))
            }
            NativeFunction { fun, .. } => fun(arguments),
        }
    }
}
