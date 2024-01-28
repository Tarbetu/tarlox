use core::fmt;
use std::{
    fmt::Display,
    io::{self},
};

#[derive(Debug, Clone)]
pub enum LoxError<'a> {
    FileError,
    UnexceptedCharacter { line: usize, character: char },
    RuntimeError { line: usize, place: &'a str },
    UnterminatedString,
    InternalParsingError(&'a str),
    Other(String),
}

impl Display for LoxError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::LoxError::*;

        match self {
            FileError => write!(f, "[Lox Error: File can't be accessed.]"),
            UnexceptedCharacter { line, character } => {
                write!(f, "[Lox Error: Unexcepted {character} at {line}]")
            }
            RuntimeError { line, place } => {
                write!(f, "[Runtime Error: Error on {} in {}]", line, place)
            }
            UnterminatedString => {
                write!(f, "[Lox Error: Unterminated String]")
            }
            InternalParsingError(msg) => write!(f, "[Internal Error: Can't parsed! {msg}]"),
            Other(txt) => write!(f, "[Unexcepted Error from io::Error - {txt}]"),
        }
    }
}

impl From<io::Error> for LoxError<'_> {
    fn from(error: io::Error) -> Self {
        use io::ErrorKind::*;

        match error.kind() {
            NotFound | PermissionDenied => Self::FileError,
            other => Self::Other(other.to_string()),
        }
    }
}
