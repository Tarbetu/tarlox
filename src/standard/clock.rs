use chrono::Utc;
use rug::Float;

use crate::{executor::LoxObject, LoxResult, NUMBER_PREC};

pub fn clock(_: &[LoxObject]) -> LoxResult<LoxObject> {
    Ok(LoxObject::from(Float::with_val(
        NUMBER_PREC,
        Utc::now().timestamp(),
    )))
}
