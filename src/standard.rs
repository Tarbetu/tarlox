mod clock;

use crate::executor::environment;
use crate::executor::Environment;
use crate::executor::LoxCallable;
use std::sync::Arc;

pub fn globals() -> Arc<Environment> {
    let env = Arc::new(Environment::new());

    environment::put_function(
        Arc::clone(&env),
        environment::variable_hash("clock"),
        LoxCallable::NativeFunction {
            arity: 0,
            fun: clock::clock,
        },
    );

    env
}
