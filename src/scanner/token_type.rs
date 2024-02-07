use rug::Float;
use std::rc::Rc;

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
    Ready,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    #[allow(clippy::upper_case_acronyms)]
    EOF,
}
