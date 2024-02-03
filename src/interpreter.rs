mod object;

use astro_float::BigFloat;
use async_recursion::async_recursion;
use parking_lot::Mutex;
use std::rc::Rc;
use std::sync::Arc;

use crate::syntax::expression::LoxLiteral;
use crate::syntax::expression::Operator;
use crate::syntax::Expression;
use object::LoxObject;

// I want to control all values by Referance Counting,
// Any since Lox is a dynamic language like Python and Ruby,
// I want to the Lox programmer don't bother with threading issues.
struct Interpreter;

impl Interpreter {
    #[async_recursion(?Send)]
    pub async fn eval(expr: Expression) -> LoxObject {
        use Expression::*;
        use LoxLiteral::*;

        match expr {
            Grouping(inner) => Self::eval(*inner).await,
            Literal(Number(n)) => {
                LoxObject::create_number(Rc::try_unwrap(n).expect("Number is still used!"))
            }
            Literal(LoxString(s)) => {
                LoxObject::create_string(Rc::try_unwrap(s).expect("Number is still used!"))
            }
            Literal(Bool(b)) => LoxObject::Boolean(b),
            Literal(Nil) => LoxObject::Nil,
            Unary(operator, right) => {
                let mut right = Self::eval(*right).await;

                match operator {
                    Operator::Minus => {
                        right.apply_negative().await;
                        right
                    }
                    Operator::Not => LoxObject::Boolean(right.is_truthy()),
                    _ => unreachable!(),
                }
            }
            _ => unimplemented!(),
        }
    }
}
