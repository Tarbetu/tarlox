mod object;

use async_recursion::async_recursion;

use crate::syntax::expression::LoxLiteral;
use crate::syntax::expression::Operator;
use crate::syntax::Expression;
use crate::syntax::Statement;
use crate::LoxResult;
use object::LoxObject;

pub async fn interpret(statements: Vec<Statement>) -> LoxResult<()> {
    for statement in statements {
        eval_statement(&statement).await?;
    }

    Ok(())
}

async fn eval_statement(stmt: &Statement) -> LoxResult<()> {
    use Statement::*;

    match stmt {
        StmtExpression(expr) => {
            eval_expression(expr).await?;

            Ok(())
        }
        Print(expr) => {
            let res = eval_expression(expr).await?;

            println!("{}", res.to_string());
            Ok(())
        }
        Ready(_expr) => {
            // If expr is a identifier, check if it's accessable
            unimplemented!()
        }
        Var(..) => unimplemented!(),
    }
}

#[async_recursion(?Send)]
async fn eval_expression(expr: &Expression) -> LoxResult<LoxObject> {
    use Expression::*;
    use LoxLiteral::*;

    match expr {
        Grouping(inner) => eval_expression(inner).await,
        // Get rid of this cloning
        Literal(Number(n)) => Ok(LoxObject::from(n)),
        Literal(LoxString(s)) => Ok(LoxObject::from(s.as_str())),
        Literal(Bool(b)) => Ok(LoxObject::from(*b)),
        Literal(Nil) => Ok(LoxObject::Nil),
        Unary(operator, right) => {
            let right = eval_expression(right).await?;

            match operator {
                Operator::Minus => Ok(LoxObject::from(right.apply_negative().await?)),
                Operator::Not => Ok(LoxObject::from(!bool::from(&right))),
                _ => unreachable!(),
            }
        }
        Binary(left, operator, right) => {
            let left = &eval_expression(left).await?;
            let right = &eval_expression(right).await?;

            match operator {
                Operator::Star => left * right,
                Operator::Slash => left / right,
                Operator::Minus => left - right,
                Operator::Plus => left + right,
                Operator::Equality => Ok(left.is_equal(&right).await),
                Operator::NotEqual => Ok(left.is_not_equal(&right).await),
                Operator::Greater => left.is_greater(&right).await,
                Operator::GreaterOrEqual => left.is_greater_equal(&right).await,
                Operator::Smaller => left.is_less(&right).await,
                Operator::SmallerOrEqual => left.is_less_equal(&right).await,
                _ => unreachable!(),
            }
        }
    }
}
