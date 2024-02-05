use crate::{LoxError, LoxResult};
use astro_float::BigFloat;
// use parking_lot::Mutex;
use std::sync::Arc;

// use std::any::Any;

use std::ops;

#[derive(Debug, PartialEq, Eq)]
pub enum LoxObject {
    Nil,
    // Remove Any
    // UserDefined(Arc<Mutex<Box<Any + PartialEq + Eq>>>),
    Number(Arc<BigFloat>),
    LoxString(Arc<String>),
    Boolean(bool),
}

impl LoxObject {
    pub fn create_number(num: BigFloat) -> Self {
        Self::Number(Arc::new(num))
    }

    pub fn create_string(s: String) -> Self {
        Self::LoxString(Arc::new(s))
    }

    pub async fn apply_negative(&self) -> LoxResult<BigFloat> {
        if let Self::Number(n) = self {
            Ok(n.neg())
        } else {
            Err(LoxError::TypeError {
                excepted_type: "Number".into(),
            })
        }
    }

    pub async fn is_equal(&self, rhs: &LoxObject) -> LoxObject {
        use LoxObject::{Boolean, Nil};

        if let (Nil, Nil) = (self, rhs) {
            Boolean(true)
        } else if let Nil = self {
            Boolean(false)
        } else {
            Boolean(self == rhs)
        }
    }

    pub async fn is_not_equal(&self, rhs: &LoxObject) -> LoxObject {
        use LoxObject::Boolean;

        if let Boolean(b) = self.is_equal(rhs).await {
            Boolean(!b)
        } else {
            unreachable!()
        }
    }

    pub async fn is_greater(&self, rhs: &LoxObject) -> LoxResult<LoxObject> {
        use LoxObject::{Boolean, Number};

        if let (Number(l), Number(r)) = (self, rhs) {
            Ok(Boolean(l > r))
        } else {
            Err(LoxError::TypeError {
                excepted_type: "Number".into(),
            })
        }
    }

    pub async fn is_greater_equal(&self, rhs: &LoxObject) -> LoxResult<LoxObject> {
        Ok(LoxObject::Boolean(
            self.is_greater(rhs).await?.into() || self.is_equal(rhs).await.into(),
        ))
    }

    pub async fn is_less(&self, rhs: &LoxObject) -> LoxResult<LoxObject> {
        use LoxObject::{Boolean, Number};

        if let (Number(l), Number(r)) = (self, rhs) {
            Ok(Boolean(l < r))
        } else {
            Err(LoxError::TypeError {
                excepted_type: "Number".into(),
            })
        }
    }

    pub async fn is_less_equal(&self, rhs: &LoxObject) -> LoxResult<LoxObject> {
        use LoxObject::Boolean;
        Ok(Boolean(
            self.is_less(rhs).await?.into() || self.is_equal(rhs).await.into(),
        ))
    }
}

impl ops::Mul<LoxObject> for LoxObject {
    type Output = LoxResult<LoxObject>;

    fn mul(self, rhs: LoxObject) -> Self::Output {
        use LoxObject::Number;

        if let (Number(l), Number(r)) = (self, rhs) {
            Ok(LoxObject::Number(Arc::new(l.mul_full_prec(&r))))
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
            Ok(LoxObject::Number(Arc::new(l.div(
                &r,
                1024,
                astro_float::RoundingMode::None,
            ))))
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
            Ok(LoxObject::Number(Arc::new(l.sub_full_prec(&r))))
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

        if let (Number(l), Number(r)) = (&self, &rhs) {
            Ok(LoxObject::Number(Arc::new(l.add_full_prec(r))))
        } else if let (LoxString(l), LoxString(r)) = (self, rhs) {
            Ok(LoxObject::LoxString(Arc::new(format!("{l}{r}"))))
        } else {
            Err(LoxError::TypeError {
                excepted_type: "Number".into(),
            })
        }
    }
}

impl Into<bool> for LoxObject {
    fn into(self) -> bool {
        use LoxObject::*;

        !matches!(self, Nil | Boolean(false))
    }
}
