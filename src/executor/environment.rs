use dashmap::DashMap;
use either::Either::{self, Left, Right};
use rayon::ThreadPool;
use std::hash::Hasher;
use std::sync::Mutex;
use std::sync::{Arc, Condvar};

use crate::{syntax::Expression, LoxResult};

use super::eval_expression;
use super::object::LoxObject;

#[derive(Debug)]
pub enum PackagedObject {
    Pending(Mutex<bool>, Condvar),
    Ready(LoxResult<LoxObject>),
}

#[derive(Debug)]
pub struct Environment {
    pub values: DashMap<u64, PackagedObject, ahash::RandomState>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: DashMap::with_hasher(ahash::RandomState::new()),
        }
    }
}

pub fn put(environment: Arc<Environment>, workers: &ThreadPool, name: &str, expr: &Expression) {
    let key = variable_hash(name);
    let condvar = Condvar::new();
    environment
        .values
        .insert(key, PackagedObject::Pending(Mutex::new(false), condvar));

    workers.install(move || {
        let value = eval_expression(environment.clone(), expr);

        if let PackagedObject::Pending(mtx, cdv) = environment.values.get(&key).unwrap().value() {
            *mtx.lock().unwrap() = true;
            cdv.notify_all();
        }

        environment.values.insert(key, PackagedObject::Ready(value));
    });
}

pub fn put_immediately(
    environment: Arc<Environment>,
    name: &str,
    expr_or_obj: Either<&Expression, LoxObject>,
) {
    Arc::clone(&environment).values.insert(
        variable_hash(name),
        PackagedObject::Ready(match expr_or_obj {
            Left(expr) => eval_expression(environment, expr),
            Right(obj) => Ok(obj),
        }),
    );
}

pub fn variable_hash(name: &str) -> u64 {
    let mut hasher = ahash::AHasher::default();
    hasher.write(name.as_bytes());
    hasher.finish()
}
