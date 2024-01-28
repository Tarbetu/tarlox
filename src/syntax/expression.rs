use std::fmt::Display;
use std::rc::Rc;

use astro_float::BigFloat;

use crate::{LoxError, Token, TokenType};

#[derive(Debug)]
pub enum Expression {
    Binary(Box<Expression>, Operator, Box<Expression>),
    Unary(Operator, Box<Expression>),
    Grouping(Box<Expression>),
    Literal(LoxLiteral),
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
        }
    }
}

#[derive(Debug)]
pub enum LoxLiteral {
    Nil,
    Number(Rc<BigFloat>),
    LoxString(Rc<String>),
    Bool(bool),
}

impl<'a> Display for LoxLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LoxLiteral::*;

        match self {
            Nil => write!(f, "Nil"),
            LoxString(s) => write!(f, "{s}"),
            Number(n) => write!(f, "{n}"),
            Bool(b) => write!(f, "{b}"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Operator {
    Equal,
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
}

// Not your best solution
impl TryInto<Operator> for &Token {
    type Error = LoxError<'static>;

    fn try_into(self) -> Result<Operator, Self::Error> {
        match self.kind {
            TokenType::Equal => Ok(Operator::Equal),
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
            _ => Err(LoxError::InternalParsingError(
                "Unmatched TokenType for operator",
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
                Equal => "==",
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
            }
        )
    }
}
