mod clock;

use crate::executor::{environment, Environment, LoxCallable, LoxObject};
use std::sync::Arc;

use either::Either;

macro_rules! make_function {
    ($env:expr, $arity:expr, $name:ident) => {
        environment::put_immediately(
            Arc::clone(&$env),
            stringify!($name),
            Either::Right(LoxObject::from(LoxCallable::NativeFunction {
                arity: $arity,
                fun: $name::$name,
            })),
        )
    };
}

pub fn globals() -> Arc<Environment> {
    let env = Arc::new(Environment::new());

    make_function!(env, 0, clock);

    env
}
