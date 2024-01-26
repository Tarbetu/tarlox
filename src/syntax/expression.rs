use std::fmt::Display;

use astro_float::BigFloat;

use crate::Token;

#[derive(Debug)]
pub enum Expression<'a> {
    Binary(&'a Expression<'a>, Token, &'a Expression<'a>),
    Unary(Token, &'a Expression<'a>),
    Grouping(&'a Expression<'a>),
    Literal(LoxLiteral),
}

#[derive(Debug)]
pub enum LoxLiteral {
    Nil,
    Number(BigFloat),
    LoxString(String),
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

impl<'a> Display for Expression<'a> {
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
