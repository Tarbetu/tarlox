use core::fmt;
use std::{
    error::Error,
    fmt::Display,
    io::{self},
};

/// This enum represents the state that we can't continue and
/// nothing is recueable
/// We should stop the program and deliver the error message to the user quickly
/// after creating the LoxError object.
#[derive(Debug, Clone)]
pub enum LoxError {
    FileError,
    UnexceptedCharacter { line: usize, character: char },
    ParseError { line: Option<usize>, msg: String },
    RuntimeError { line: Option<usize>, msg: String },
    UnterminatedString,
    InternalParsingError(String),
    ExceptedExpression(usize),
    TypeError { excepted_type: String },
    Other(String),
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
                    write!(f, "[Runtime Error: Error at {l}. {msg}]")
                } else {
                    write!(f, "[Runtime Error: Error at end. {msg}]")
                }
            }
            ParseError { line, msg } => {
                if let Some(l) = line {
                    write!(f, "[Parse Error: Error at {l}. {msg}]")
                } else {
                    write!(f, "[Parse Error: Error at end. {msg}]")
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
            InternalParsingError(msg) => write!(f, "[Internal Error: Can't parsed! {msg}]"),
            Other(txt) => write!(f, "[Unexcepted Error from io::Error - {txt}]"),
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
