use astro_float::BigFloat;
use parking_lot::Mutex;
use std::sync::Arc;

use std::any::Any;

pub enum LoxObject {
    Nil,
    // Remove Any
    UserDefined(Arc<Mutex<Box<dyn Any>>>),
    Number(Arc<Mutex<BigFloat>>),
    LoxString(Arc<Mutex<String>>),
    Boolean(bool),
}

impl LoxObject {
    pub fn create_number(num: BigFloat) -> Self {
        Self::Number(Arc::new(Mutex::new(num)))
    }

    pub fn create_string(s: String) -> Self {
        Self::LoxString(Arc::new(Mutex::new(s)))
    }

    pub fn is_truthy(&self) -> bool {
        use LoxObject::*;

        match self {
            Nil | Boolean(false) => false,
            _ => true,
        }
    }

    pub async fn apply_negative(&mut self) {
        if let Self::Number(n) = self {
            let aref = n.clone();
            let mut val = aref.lock();

            *val = val.neg();
        } else {
            panic!("Excepted number!")
        }
    }
}
