use std::hash::Hash;

use rug::Float;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // one or two character tokens,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // literals,
    Identifier(String),
    LoxString(String),
    Number(Float),
    // keywords,
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    IsReady,
    Return,
    Super,
    This,
    True,
    Var,
    AwaitVar,
    While,

    #[allow(clippy::upper_case_acronyms)]
    EOF,
}

impl Hash for TokenType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use TokenType::*;

        match self {
            Identifier(str) => format!("IDENT_{str}").hash(state),
            LoxString(str) => format!("STR_{str}").hash(state),
            other => format!("{:?}", other).hash(state),
        }
    }
}
