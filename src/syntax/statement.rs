use crate::Token;

use super::Expression;

#[derive(Debug)]
pub enum Statement {
    Print(Expression),
    StmtExpression(Expression),
    Var(Token, Option<Expression>),
}
