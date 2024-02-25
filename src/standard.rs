mod clock;

use crate::executor::environment;
use crate::executor::Environment;
use crate::executor::LoxCallable;
use std::sync::Arc;

macro_rules! make_function {
    ($env:expr, $arity:expr, $name:ident) => {
        environment::put_function(
            Arc::clone(&$env),
            environment::env_hash(stringify!($name)),
            LoxCallable::NativeFunction {
                arity: $arity,
                fun: $name::$name,
            },
        )
    };
}

pub fn globals() -> Arc<Environment> {
    let env = Arc::new(Environment::new());

    make_function!(env, 0, clock);

    env
}
