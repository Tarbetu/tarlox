use ahash::AHashMap;
use either::Either::{self, Left, Right};
use parking_lot::Mutex;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::sync::Arc;

use crate::{syntax::Expression, LoxResult};

use super::eval_expression;
use super::object::LoxObject;

use std::{
    num::NonZeroUsize,
    sync::mpsc::{channel, Receiver},
    thread::available_parallelism,
};

#[derive(Debug)]
pub enum PackagedObject {
    Pending(Receiver<LoxResult<LoxObject>>),
    Ready(LoxResult<LoxObject>),
}

#[derive(Debug)]
pub struct Environment {
    pub values: AHashMap<String, PackagedObject>,
    pub pool: ThreadPool,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: AHashMap::new(),
            pool: ThreadPoolBuilder::new()
                .num_threads(
                    available_parallelism()
                        .unwrap_or(NonZeroUsize::new(1).unwrap())
                        .into(),
                )
                .build()
                .unwrap(),
        }
    }
}

pub fn put(environment: Arc<Mutex<Environment>>, name: String, expr: &Expression) {
    let (sender, receiver) = channel();

    let environment_inner = environment.clone();
    let mut env = environment.lock();
    env.values.insert(name, PackagedObject::Pending(receiver));

    env.pool.install(move || {
        sender
            .send(eval_expression(environment_inner, expr))
            .unwrap();
    });
}

pub fn put_immediately(
    environment: Arc<Mutex<Environment>>,
    name: String,
    expr_or_obj: Either<&Expression, LoxObject>,
) {
    environment.clone().lock().values.insert(
        name,
        PackagedObject::Ready(match expr_or_obj {
            Left(expr) => eval_expression(environment, expr),
            Right(obj) => Ok(obj),
        }),
    );
}
