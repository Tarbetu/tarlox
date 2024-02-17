use dashmap::DashMap;
use either::Either::{self, Left, Right};
use rayon::ThreadPool;
use std::hash::Hasher;
use std::sync::Mutex;
use std::sync::{Arc, Condvar};

use super::eval_expression;
use super::function::LoxCallable;
use super::object::LoxObject;
use crate::syntax::Expression;
use crate::LoxResult;

pub enum PackagedObject {
    Pending(Mutex<bool>, Condvar),
    Ready(LoxResult<Either<LoxObject, LoxCallable>>),
}

pub struct Environment {
    pub enclosing: Option<Arc<Environment>>,
    pub values: DashMap<u64, PackagedObject, ahash::RandomState>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: DashMap::with_hasher(ahash::RandomState::new()),
            enclosing: None,
        }
    }

    pub fn new_with_parent(enclosing: Arc<Environment>) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: DashMap::with_hasher(ahash::RandomState::new()),
        }
    }

    pub fn get(
        &self,
        key: &u64,
    ) -> Option<dashmap::mapref::one::Ref<'_, u64, PackagedObject, ahash::RandomState>> {
        self.values.get(key).or(match &self.enclosing {
            Some(env) => env.get(key),
            None => None,
        })
    }

    pub fn remove(&self, key: &u64) -> Option<(u64, PackagedObject)> {
        self.values.remove(key).or(match &self.enclosing {
            Some(env) => env.remove(key),
            None => None,
        })
    }
}

macro_rules! create_sub_environment {
    ($existing_key:expr, $env:expr) => {
        match $existing_key {
            Some((key, value)) => {
                let new_map = DashMap::with_hasher(ahash::RandomState::new());
                new_map.insert(key, value);

                Environment {
                    enclosing: Some(Arc::clone(&$env.clone())),
                    values: new_map,
                }
                .into()
            }
            None => $env.clone(),
        }
    };
}

pub fn put(environment: Arc<Environment>, workers: &ThreadPool, name: &str, expr: &Expression) {
    let key = variable_hash(name);

    // To avoid deadlock, we have to remove the old value
    let existing_key = environment.values.remove(&key);

    let condvar = Condvar::new();

    environment
        .values
        .insert(key, PackagedObject::Pending(Mutex::new(false), condvar));

    let sub_environment = create_sub_environment!(existing_key, environment);

    workers.install(move || {
        let value = eval_expression(Arc::clone(&sub_environment), expr).map(Either::Left);

        if let PackagedObject::Pending(mtx, cdv) = sub_environment.get(&key).unwrap().value() {
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
    let key = variable_hash(name);
    // To avoid deadlock, we have to remove the old value
    let existing_key = environment.values.remove(&key);

    let sub_environment = create_sub_environment!(existing_key, environment);

    Arc::clone(&environment).values.insert(
        variable_hash(name),
        PackagedObject::Ready(match expr_or_obj {
            Left(expr) => eval_expression(sub_environment, expr).map(Either::Left),
            Right(obj) => Ok(Either::Left(obj)),
        }),
    );
}

pub fn put_function(environment: Arc<Environment>, key: u64, fun: LoxCallable) {
    // Analyze function if not native
    // So check if it's recursive
    // let is_recursive = false;

    environment
        .values
        .insert(key, PackagedObject::Ready(Ok(Either::Right(fun))));
}

pub fn variable_hash(name: &str) -> u64 {
    let mut hasher = ahash::AHasher::default();
    hasher.write(name.as_bytes());
    hasher.finish()
}

pub fn function_hash(name: &str) -> u64 {
    let mut hasher = ahash::AHasher::default();
    hasher.write("@".as_bytes());
    hasher.write(name.as_bytes());
    hasher.finish()
}
