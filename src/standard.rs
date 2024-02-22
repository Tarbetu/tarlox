use crate::executor::environment;
use crate::executor::Environment;
use std::sync::Arc;

fn globals() -> Arc<Environment> {
    let env = Arc::new(Environment::new());

    environment::put_function(
        env,
        environment::variable_hash(unimplemented!()),
        unimplemented!(),
    );

    env
}
