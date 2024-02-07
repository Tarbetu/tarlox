mod object;

use async_recursion::async_recursion;
use std::rc::Rc;

use crate::syntax::expression::LoxLiteral;
use crate::syntax::expression::Operator;
use crate::syntax::Expression;
use crate::LoxResult;
use object::LoxObject;

pub struct Interpreter;

impl Interpreter {
    pub async fn interpret(expr: Expression) {
        let res = Self::eval(expr).await;

        if let Ok(obj) = res {
            println!("{}", obj.to_string());
        }
    }

    #[async_recursion(?Send)]
    async fn eval(expr: Expression) -> LoxResult<LoxObject> {
        use Expression::*;
        use LoxLiteral::*;

        match expr {
            Grouping(inner) => Self::eval(*inner).await,
            Literal(Number(n)) => Ok(LoxObject::from(
                Rc::try_unwrap(n).expect("Number is still used!"),
            )),
            Literal(LoxString(s)) => Ok(LoxObject::from(
                Rc::try_unwrap(s).expect("Number is still used!"),
            )),
            Literal(Bool(b)) => Ok(LoxObject::from(b)),
            Literal(Nil) => Ok(LoxObject::Nil),
            Unary(operator, right) => {
                let right = Self::eval(*right).await?;

                match operator {
                    Operator::Minus => Ok(LoxObject::from(right.apply_negative().await?)),
                    Operator::Not => Ok(LoxObject::from(!bool::from(right))),
                    _ => unreachable!(),
                }
            }
            Binary(left, operator, right) => {
                let left = Self::eval(*left).await?;
                let right = Self::eval(*right).await?;

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
}
