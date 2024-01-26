use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: super::TokenType,
    pub line: usize,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]{:?}", self.line, self.kind)
    }
}
