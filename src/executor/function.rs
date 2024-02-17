use crate::{syntax::Statement, LoxError, LoxResult, Token};

use super::{object::LoxObject, Executor};

pub enum LoxCallable {
    Function {
        parameters: Vec<Token>,
        body: Statement,
        executor: Executor,
        is_recursive: bool,
    },
    NativeFunction {
        arity: usize,
        fun: fn(&[LoxObject]) -> LoxResult<()>,
    },
}

impl LoxCallable {
    fn arity(&self) -> usize {
        use LoxCallable::*;

        match self {
            Function { parameters, .. } => parameters.len(),
            NativeFunction { arity, .. } => *arity,
        }
    }

    pub fn call(&self, arguments: &[LoxObject]) -> LoxResult<()> {
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
                    executor.eval_statement(body)
                }
            }
            NativeFunction { fun, .. } => fun(arguments),
        }
    }
}
