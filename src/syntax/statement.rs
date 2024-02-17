use crate::Token;

use super::Expression;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Print(Expression),
    StmtExpression(Expression),
    Var(Token, Option<Expression>),
    AwaitVar(Token, Expression),
    Block(Vec<Statement>),
    // Condition        If Branch      Else Branch
    If(Expression, Box<Statement>, Option<Box<Statement>>),
    //     Condition     Body
    While(Expression, Box<Statement>),
    //        Name     Params      Body
    Function(Token, Vec<Token>, Box<Statement>),
}
