use core::fmt;
use std::{
    fmt::Display,
    io::{self},
};

// We use static because this enum created while interpreter terminated
#[derive(Debug, Clone)]
pub enum LoxError {
    FileError,
    UnexceptedCharacter {
        line: usize,
        character: char,
    },
    ParseError {
        line: Option<usize>,
        msg: &'static str,
    },
    RuntimeError {
        line: Option<usize>,
        msg: &'static str,
    },
    UnterminatedString,
    InternalParsingError(&'static str),
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
                    write!(f, "[Runtime Error: Error at {}. {}]", l, msg)
                } else {
                    write!(f, "[Runtime Error: Error at end. {}]", msg)
                }
            }
            ParseError { line, msg } => {
                if let Some(l) = line {
                    write!(f, "[Parse Error: Error at {}. {}]", l, msg)
                } else {
                    write!(f, "[Parse Error: Error at end. {}]", msg)
                }
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
