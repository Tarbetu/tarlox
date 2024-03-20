use std::fmt::Display;
use std::hash::Hash;

use rand::random;

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    pub id: usize,
    pub kind: super::TokenType,
    pub line: usize,
}

impl Token {
    pub fn new(kind: super::TokenType, line: usize) -> Token {
        Token {
            id: random(),
            kind,
            line,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

impl Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.line.hash(state);
        self.kind.hash(state);
    }
}

impl Eq for Token {}
