mod clock;

use crate::executor::{environment, Environment, LoxCallable, LoxObject};
use std::sync::Arc;

use dashmap::DashMap;
use either::Either;

macro_rules! make_function {
    ($env:expr, $locals:expr, $arity:expr, $name:ident) => {
        environment::put_immediately(
            Arc::clone(&$env),
            Arc::clone(&$locals),
            stringify!($name),
            Either::Right(LoxObject::from(LoxCallable::NativeFunction {
                arity: $arity,
                fun: $name::$name,
            })),
        )
    };
}

pub fn globals() -> Arc<Environment> {
    let env = Arc::new(Environment::default());
    let locals = Arc::new(DashMap::with_hasher(ahash::RandomState::new()));

    make_function!(env, locals, 0, clock);

    env
}
