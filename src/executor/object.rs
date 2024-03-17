use crate::{LoxError, LoxResult, Token, TokenType, NUMBER_PREC};

use dashmap::DashMap;
use rug::Float;
use std::sync::Arc;

use std::ops;

use super::class::LoxClass;
use super::LoxCallable;

#[derive(Debug)]
pub enum LoxObject {
    Nil,
    Instance(
        u64,
        Arc<LoxClass>,
        Arc<DashMap<String, LoxObject, ahash::RandomState>>,
    ),
    Number(Arc<Float>),
    LoxString(Arc<String>),
    Boolean(bool),
    Callable(Arc<LoxCallable>),
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

    pub fn get(&self, method: &Token) -> LoxResult<LoxObject> {
        if let (LoxObject::Instance(.., class, fields), TokenType::Identifier(name)) =
            (self, &method.kind)
        {
            match (fields.get(name), class.find_method(name)) {
                (Some(val), ..) => Ok(LoxObject::from(val.value())),
                (None, Some(callable)) => Ok(LoxObject::from(callable.bind(self))),
                (None, None) => Err(LoxError::RuntimeError {
                    line: Some(method.line),
                    msg: format!("Undefined property {name}."),
                }),
            }
        } else {
            Err(LoxError::RuntimeError {
                line: Some(method.line),
                msg: "Only instances have properties".into(),
            })
        }
    }

    pub fn set(&self, method: &Token, value: LoxObject) -> LoxResult<LoxObject> {
        if let (LoxObject::Instance(.., fields), TokenType::Identifier(name)) = (self, &method.kind)
        {
            fields.insert(name.to_owned(), value.clone());

            Ok(value)
        } else {
            // Executor already checks that
            unreachable!()
        }
    }
}

impl PartialEq for LoxObject {
    fn eq(&self, other: &Self) -> bool {
        use LoxObject::*;

        match (self, other) {
            (Nil, Nil) => true,
            (Instance(id, ..), Instance(other_id, ..)) => id == other_id,
            (Number(num), Number(other_num)) => num == other_num,
            (LoxString(str), LoxString(other_str)) => str == other_str,
            (Boolean(bool), Boolean(other_bool)) => bool == other_bool,
            (Callable(callable), Callable(other_callable)) => callable == other_callable,
            _ => false,
        }
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

        if let (LoxString(l), r) = (&self, &rhs) {
            Ok(LoxObject::from(format!("{}{}", l, r.to_string()).as_str()))
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

impl From<LoxCallable> for LoxObject {
    fn from(value: LoxCallable) -> Self {
        Self::Callable(Arc::new(value))
    }
}

impl From<&LoxCallable> for LoxObject {
    fn from(value: &LoxCallable) -> Self {
        Self::Callable(Arc::new(LoxCallable::from(value)))
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
            Callable(callable) => {
                if let LoxCallable::Class { class } = callable.as_ref() {
                    format!("#<class {}>", class)
                } else {
                    format!("<fun arity: {}>", callable.arity())
                }
            }
            Instance(id, class, ..) => format!("#<{} instance as {}>", class.name, id),
        }
    }
}

impl ToString for &LoxObject {
    fn to_string(&self) -> String {
        LoxObject::from(*self).to_string()
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
            Callable(callable) => Callable(Arc::clone(callable)),
            Instance(id, class, fields, ..) => Instance(*id, Arc::clone(class), Arc::clone(fields)),
        }
    }
}

impl Clone for LoxObject {
    fn clone(&self) -> Self {
        LoxObject::from(self)
    }
}
