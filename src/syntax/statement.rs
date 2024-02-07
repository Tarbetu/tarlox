use super::Expression;

#[derive(Debug)]
pub enum Statement {
    Print(Expression),
    Ready(Expression),
    StmtExpression(Expression),
}
