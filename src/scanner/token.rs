use std::fmt::Display;
use std::hash::Hash;

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    pub kind: super::TokenType,
    pub line: usize,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]{:?}", self.line, self.kind)
    }
}

impl Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.line.hash(state);
        self.kind.hash(state);
    }
}
