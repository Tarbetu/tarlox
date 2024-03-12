use std::fmt::Display;
use std::sync::Arc;

use rug::Float;

use super::Statement;
use crate::{LoxError, Token, TokenType};

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Expression {
    Binary(Box<Expression>, Operator, Box<Expression>),
    Unary(Operator, Box<Expression>),
    Grouping(Box<Expression>),
    Literal(LoxLiteral),
    Logical(Box<Expression>, Operator, Box<Expression>),
    Variable(Token),
    Assign(Token, Box<Expression>),
    Call(Box<Expression>, Token, Vec<Expression>),
    Lambda(Vec<Token>, Arc<Statement>),
    Get(Box<Expression>, Token),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Expression::*;

        match self {
            Binary(left, operator, right) => {
                write!(f, "({operator} {left} {right})")
            }
            Unary(operator, right) => {
                write!(f, "({operator} {right})")
            }
            Grouping(inner) => {
                write!(f, "(grouping {inner})")
            }
            Literal(literal) => {
                write!(f, "{literal}")
            }
            Variable(token) => {
                write!(f, "(var {token})")
            }
            Assign(token, value) => {
                write!(f, "(assign {token} {value})")
            }
            Logical(left, operator, right) => {
                write!(f, "({operator} {left} {right})")
            }
            Call(callee, _paren, arguments) => {
                write!(f, "({callee} #{arguments:?})")
            }
            Lambda(params, _body) => {
                write!(f, "<lambda arity: {}>", params.len())
            }
            Get(object, name) => {
                write!(f, "({object} {name})")
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LoxLiteral {
    Nil,
    Number(Float),
    LoxString(String),
    Bool(bool),
}

impl Display for LoxLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LoxLiteral::*;

        match self {
            Nil => write!(f, "nil"),
            LoxString(s) => write!(f, r#""{s}""#),
            Number(n) => write!(f, "{n}"),
            Bool(b) => write!(f, "{b}"),
        }
    }
}

impl std::hash::Hash for LoxLiteral {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use LoxLiteral::*;

        match self {
            Nil => "NIL_LIT".hash(state),
            LoxString(str) => format!("LOX_LIT_STR_{str}").hash(state),
            Bool(b) => format!("BOOL_LIT_{b}").hash(state),
            Number(float) => format!("NUMBER_LIT_{float}").hash(state),
        }
    }
}

impl Eq for LoxLiteral {}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum Operator {
    Equality,
    NotEqual,
    Assignment,
    Minus,
    Plus,
    Star,
    Slash,
    Not,
    Smaller,
    SmallerOrEqual,
    Greater,
    GreaterOrEqual,
    IsReady,
    Or,
    And,
}

// Not your best solution
impl TryInto<Operator> for &Token {
    type Error = LoxError;

    fn try_into(self) -> Result<Operator, Self::Error> {
        match self.kind {
            TokenType::EqualEqual => Ok(Operator::Equality),
            TokenType::BangEqual => Ok(Operator::NotEqual),
            TokenType::Minus => Ok(Operator::Minus),
            TokenType::Plus => Ok(Operator::Plus),
            TokenType::Star => Ok(Operator::Star),
            TokenType::Slash => Ok(Operator::Slash),
            TokenType::Bang => Ok(Operator::Not),
            TokenType::Greater => Ok(Operator::Greater),
            TokenType::Less => Ok(Operator::Smaller),
            TokenType::GreaterEqual => Ok(Operator::GreaterOrEqual),
            TokenType::LessEqual => Ok(Operator::SmallerOrEqual),
            TokenType::IsReady => Ok(Operator::IsReady),
            TokenType::And => Ok(Operator::And),
            TokenType::Or => Ok(Operator::Or),
            _ => Err(LoxError::InternalError(
                "Unmatched TokenType for operator".into(),
            )),
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Operator::*;

        write!(
            f,
            "{}",
            match *self {
                Equality => "==",
                NotEqual => "!=",
                Assignment => "=",
                Minus => "-",
                Plus => "+",
                Star => "*",
                Slash => "/",
                Not => "!",
                Smaller => "<",
                SmallerOrEqual => "<=",
                Greater => ">",
                GreaterOrEqual => ">=",
                IsReady => "is_ready",
                And => "and",
                Or => "or",
            }
        )
    }
}
