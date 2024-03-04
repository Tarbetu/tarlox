use core::fmt;
use std::{
    fmt::Display,
    io::{self},
    sync::Arc,
};

use crate::executor::Environment;
use crate::syntax::Expression;

#[derive(Debug)]
pub enum LoxError {
    FileError,
    UnexceptedCharacter { line: usize, character: char },
    ParseError { line: Option<usize>, msg: String },
    RuntimeError { line: Option<usize>, msg: String },
    UnterminatedString,
    InternalError(String),
    ExceptedExpression(usize),
    TypeError { excepted_type: String },
    Other(String),
    Return(Arc<Environment>, Option<Arc<Expression>>),
}

impl Display for LoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::LoxError::*;

        match self {
            FileError => write!(f, "[Lox Error: File can't be accessed.]"),
            UnexceptedCharacter { line, character } => {
                write!(f, "[Lox Error: Unexcepted {character} at {line}]")
            }
            RuntimeError { line, msg } => {
                if let Some(l) = line {
                    write!(f, "[Runtime Error: Error at {l} - {msg}]")
                } else {
                    write!(f, "[Runtime Error: Error at end - {msg}]")
                }
            }
            ParseError { line, msg } => {
                if let Some(l) = line {
                    write!(f, "[Parse Error: Error at {l} - {msg}]")
                } else {
                    write!(f, "[Parse Error: Error at end - {msg}]")
                }
            }
            ExceptedExpression(line) => {
                write!(
                    f,
                    "[Parse Error: Excepted Expression, found nothing ({line})]"
                )
            }
            TypeError { excepted_type } => {
                write!(f, "[Type Error: Excepted {excepted_type}]")
            }
            UnterminatedString => {
                write!(f, "[Lox Error: Unterminated String]")
            }
            InternalError(msg) => write!(f, "[Internal Error: {msg}]"),
            Other(txt) => write!(f, "[Unexcepted Error from io::Error - {txt}]"),
            Return(..) => write!(f, "Unhandled return statement."),
        }
    }
}

impl From<io::Error> for LoxError {
    fn from(error: io::Error) -> Self {
        use io::ErrorKind::*;

        match error.kind() {
            NotFound | PermissionDenied => Self::FileError,
            other => Self::Other(other.to_string()),
        }
    }
}

impl From<&LoxError> for LoxError {
    fn from(value: &LoxError) -> Self {
        use LoxError::*;

        // Strict copy?
        // Maybe we can prefer a Rc
        match value {
            FileError => FileError,
            UnterminatedString => UnterminatedString,
            UnexceptedCharacter { line, character } => UnexceptedCharacter {
                line: *line,
                character: *character,
            },
            ParseError { line, msg } => ParseError {
                line: *line,
                msg: msg.to_owned(),
            },
            RuntimeError { line, msg } => RuntimeError {
                line: *line,
                msg: msg.to_owned(),
            },
            InternalError(str) => InternalError(str.to_owned()),
            ExceptedExpression(line) => ExceptedExpression(*line),
            TypeError { excepted_type } => TypeError {
                excepted_type: excepted_type.to_owned(),
            },
            Other(str) => Other(str.to_owned()),
            Return(env, expr) => Return(Arc::clone(env), expr.as_ref().map(Arc::clone)),
        }
    }
}
