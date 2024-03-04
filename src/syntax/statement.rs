use crate::Token;
use std::{hash::Hash, sync::Arc};

use super::Expression;

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Print(Expression),
    StmtExpression(Expression),
    Var(Token, Option<Arc<Expression>>),
    AwaitVar(Token, Expression),
    Block(Arc<Vec<Arc<Statement>>>),
    // Condition        If Branch      Else Branch
    If(Expression, Arc<Statement>, Option<Arc<Statement>>),
    //     Condition     Body
    While(Expression, Arc<Statement>),
    //        Name     Params      Body
    Function(Token, Vec<Token>, Arc<Statement>),
    Return(Option<Arc<Expression>>),
}

impl Hash for Statement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        format!("{:?}", self).hash(state);
    }
}
