use crate::{LoxError, TokenType};

/// This is a error class for internal parser errors
/// This can be rescued or converted to the a LoxError
#[derive(Debug, Clone)]
pub enum MinorParserError {
    // TypeError,
    Unmatched(TokenType),
}

impl MinorParserError {
    pub fn into_lox_error(
        self,
        line: usize,
        _excepted: Option<&str>,
        _found: Option<&str>,
    ) -> LoxError {
        use MinorParserError::*;

        match self {
            // TypeError => LoxError::RuntimeError {
            //     line: Some(line),
            //     msg: format!(
            //         "Unmatched Type. Excepted {}, found {}",
            //         excepted.unwrap(),
            //         found.unwrap()
            //     ),
            // },
            Unmatched(token_type) => LoxError::ParseError {
                line: Some(line),
                msg: format!("{:?} not found", token_type),
            },
        }
    }
}
