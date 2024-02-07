use crate::{LoxError, LoxResult};

use rug::Float;
// use ahash::AHashMap;
// use std::sync::Arc;

// use std::any::Any;

use std::ops;

#[derive(Debug, PartialEq)]
pub enum LoxObject {
    Nil,
    // UserDefined(AHashMap<String, LoxObject>),
    Number(Float),
    LoxString(String),
    Boolean(bool),
}

impl LoxObject {
    pub async fn apply_negative(&self) -> LoxResult<Float> {
        if let Self::Number(n) = self {
            Ok(-n.clone())
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
            Self::from(!b)
        } else {
            unreachable!()
        }
    }

    pub async fn is_greater(&self, rhs: &LoxObject) -> LoxResult<LoxObject> {
        use LoxObject::Number;

        if let (Number(l), Number(r)) = (self, rhs) {
            Ok(Self::from(l > r))
        } else {
            Err(LoxError::TypeError {
                excepted_type: "Number".into(),
            })
        }
    }

    pub async fn is_greater_equal(&self, rhs: &LoxObject) -> LoxResult<LoxObject> {
        Ok(LoxObject::from(
            self.is_greater(rhs).await?.into() || self.is_equal(rhs).await.into(),
        ))
    }

    pub async fn is_less(&self, rhs: &LoxObject) -> LoxResult<LoxObject> {
        use LoxObject::Number;

        if let (Number(l), Number(r)) = (self, rhs) {
            Ok(Self::from(l < r))
        } else {
            Err(LoxError::TypeError {
                excepted_type: "Number".into(),
            })
        }
    }

    pub async fn is_less_equal(&self, rhs: &LoxObject) -> LoxResult<LoxObject> {
        Ok(Self::from(
            self.is_less(rhs).await?.into() || self.is_equal(rhs).await.into(),
        ))
    }
}

impl ops::Mul<LoxObject> for LoxObject {
    type Output = LoxResult<LoxObject>;

    fn mul(self, rhs: LoxObject) -> Self::Output {
        use LoxObject::Number;

        if let (Number(l), Number(r)) = (self, rhs) {
            Ok(LoxObject::from(l * &r))
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
            Ok(LoxObject::from(l / &r))
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
            Ok(LoxObject::from(l - &r))
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
            Ok(LoxObject::from(l.clone() + r))
        } else if let (LoxString(l), LoxString(r)) = (self, rhs) {
            Ok(LoxObject::from(format!("{l}{r}")))
        } else {
            Err(LoxError::TypeError {
                excepted_type: "Number".into(),
            })
        }
    }
}

impl From<LoxObject> for bool {
    fn from(obj: LoxObject) -> bool {
        use LoxObject::*;

        !matches!(obj, Nil | Boolean(false))
    }
}

impl From<bool> for LoxObject {
    fn from(b: bool) -> LoxObject {
        Self::Boolean(b)
    }
}

impl From<String> for LoxObject {
    fn from(s: String) -> LoxObject {
        Self::LoxString(s)
    }
}

impl From<Float> for LoxObject {
    fn from(n: Float) -> LoxObject {
        Self::Number(n)
    }
}

impl ToString for LoxObject {
    fn to_string(&self) -> String {
        use LoxObject::*;

        match self {
            Nil => String::from("nil"),
            LoxString(s) => s.to_owned(),
            Number(n) => n.to_string(),
            Boolean(b) => b.to_string(),
        }
    }
}
