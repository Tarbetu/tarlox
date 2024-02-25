use crate::Token;
use std::sync::Arc;

use super::Expression;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Print(Expression),
    StmtExpression(Expression),
    Var(Token, Option<Expression>),
    AwaitVar(Token, Expression),
    Block(Arc<Vec<Arc<Statement>>>),
    // Condition        If Branch      Else Branch
    If(Expression, Arc<Statement>, Option<Arc<Statement>>),
    //     Condition     Body
    While(Expression, Arc<Statement>),
    //        Name     Params      Body
    Function(Token, Vec<Token>, Arc<Statement>),
    Return(Option<Expression>),
}
