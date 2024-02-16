use crate::{LoxError, LoxResult, NUMBER_PREC};

use either::Either;
use rug::Float;
use std::sync::Arc;

// use std::any::Any;

use std::ops;

use super::function::LoxFunction;

#[derive(Debug, PartialEq)]
pub enum LoxObject {
    Nil,
    // UserDefined(AHashMap<String, LoxObject>),
    Number(Arc<Float>),
    LoxString(Arc<String>),
    Boolean(bool),
    FunctionId(u64),
}

impl LoxObject {
    pub fn apply_negative(self) -> LoxResult<Float> {
        if let Self::Number(n) = self {
            Ok((*n.as_neg()).to_owned())
        } else {
            Err(LoxError::TypeError {
                excepted_type: "Number".into(),
            })
        }
    }

    pub fn is_equal(&self, rhs: &LoxObject) -> LoxObject {
        use LoxObject::{Boolean, Nil};

        if let (Nil, Nil) = (self, rhs) {
            Boolean(true)
        } else if let Nil = self {
            Boolean(false)
        } else {
            Boolean(*self == *rhs)
        }
    }

    pub fn is_not_equal(&self, rhs: &LoxObject) -> LoxObject {
        use LoxObject::Boolean;

        if let Boolean(b) = self.is_equal(rhs) {
            Self::from(!b)
        } else {
            unreachable!()
        }
    }

    pub fn is_greater(&self, rhs: &LoxObject) -> LoxResult<LoxObject> {
        use LoxObject::Number;

        if let (Number(l), Number(r)) = (self, rhs) {
            Ok(Self::from(l > r))
        } else {
            Err(LoxError::TypeError {
                excepted_type: "Number".into(),
            })
        }
    }

    pub fn is_greater_equal(&self, rhs: &LoxObject) -> LoxResult<LoxObject> {
        Ok(LoxObject::from(
            bool::from(&self.is_greater(rhs)?) || bool::from(&self.is_equal(rhs)),
        ))
    }

    pub fn is_less(&self, rhs: &LoxObject) -> LoxResult<LoxObject> {
        use LoxObject::Number;

        if let (Number(l), Number(r)) = (self, rhs) {
            Ok(Self::from(l < r))
        } else {
            Err(LoxError::TypeError {
                excepted_type: "Number".into(),
            })
        }
    }

    pub fn is_less_equal(&self, rhs: &LoxObject) -> LoxResult<LoxObject> {
        Ok(Self::from(
            bool::from(&self.is_less(rhs)?) || bool::from(&self.is_equal(rhs)),
        ))
    }
}

impl ops::Mul<LoxObject> for LoxObject {
    type Output = LoxResult<LoxObject>;

    fn mul(self, rhs: LoxObject) -> Self::Output {
        use LoxObject::Number;

        if let (Number(l), Number(r)) = (self, rhs) {
            Ok(LoxObject::from(Float::with_val(NUMBER_PREC, &*l * &*r)))
        } else {
            Err(LoxError::TypeError {
                excepted_type: "Number".into(),
            })
        }
    }
}

impl ops::Div<LoxObject> for LoxObject {
    type Output = LoxResult<LoxObject>;

    fn div(self, rhs: LoxObject) -> Self::Output {
        use LoxObject::Number;

        if let (Number(l), Number(r)) = (self, rhs) {
            Ok(LoxObject::from(Float::with_val(NUMBER_PREC, &*l / &*r)))
        } else {
            Err(LoxError::TypeError {
                excepted_type: "Number".into(),
            })
        }
    }
}

impl ops::Sub<LoxObject> for LoxObject {
    type Output = LoxResult<LoxObject>;

    fn sub(self, rhs: LoxObject) -> Self::Output {
        use LoxObject::Number;

        if let (Number(l), Number(r)) = (self, rhs) {
            Ok(LoxObject::from(Float::with_val(NUMBER_PREC, &*l - &*r)))
        } else {
            Err(LoxError::TypeError {
                excepted_type: "Number".into(),
            })
        }
    }
}

impl ops::Add<LoxObject> for LoxObject {
    type Output = LoxResult<LoxObject>;

    fn add(self, rhs: LoxObject) -> Self::Output {
        use LoxObject::{LoxString, Number};

        if let (LoxString(l), LoxString(r)) = (&self, &rhs) {
            Ok(LoxObject::from(format!("{l}{r}").as_str()))
        } else if let (Number(l), Number(r)) = (self, rhs) {
            Ok(LoxObject::from(Float::with_val(NUMBER_PREC, &*l + &*r)))
        } else {
            Err(LoxError::TypeError {
                excepted_type: "Number".into(),
            })
        }
    }
}

impl From<&LoxObject> for bool {
    fn from(obj: &LoxObject) -> bool {
        use LoxObject::*;

        !matches!(obj, Nil | Boolean(false))
    }
}

impl From<bool> for LoxObject {
    fn from(b: bool) -> LoxObject {
        Self::Boolean(b)
    }
}

impl From<&str> for LoxObject {
    fn from(s: &str) -> LoxObject {
        Self::LoxString(Arc::new(s.into()))
    }
}

impl From<Float> for LoxObject {
    fn from(n: Float) -> LoxObject {
        Self::Number(n.into())
    }
}

impl From<&Float> for LoxObject {
    fn from(n: &Float) -> LoxObject {
        Self::Number(n.to_owned().into())
    }
}

impl ToString for LoxObject {
    fn to_string(&self) -> String {
        use LoxObject::*;

        match self {
            Nil => String::from("nil"),
            LoxString(s) => String::clone(s),
            Number(n) => {
                if **n == Float::with_val(NUMBER_PREC, 0) {
                    String::from('0')
                } else {
                    let result = n.to_string();
                    let result = result.trim_end_matches('0').trim_end_matches('.');
                    result.to_string()
                }
            }
            Boolean(b) => b.to_string(),
            FunctionId(callable_hash) => format!("<fonk {callable_hash}>"),
        }
    }
}

impl From<&LoxObject> for LoxObject {
    fn from(value: &LoxObject) -> Self {
        use LoxObject::*;

        match value {
            LoxString(str) => LoxString(Arc::clone(str)),
            Number(num) => Number(Arc::clone(num)),
            Boolean(bool) => Boolean(*bool),
            Nil => Nil,
            FunctionId(callable_hash) => FunctionId(*callable_hash),
        }
    }
}
