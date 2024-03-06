use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use either::Either::{self, Left, Right};
use std::hash::Hasher;
use std::sync::Mutex;
use std::sync::{Arc, Condvar};

use super::object::LoxObject;
use super::{Executor, LocalsMap};
use crate::syntax::Expression;
use crate::{LoxResult, WORKERS};

#[derive(Debug)]
pub enum PackagedObject {
    Pending(Mutex<bool>, Condvar),
    Ready(LoxResult<LoxObject>),
}

impl PackagedObject {
    pub fn wait_for_value(&self) -> &LoxResult<LoxObject> {
        match self {
            Self::Pending(mtx, cvar) => {
                let res = mtx.lock().unwrap();

                let _ = cvar.wait_while(res, |pending| !*pending);
                self.wait_for_value()
            }
            Self::Ready(val) => val,
        }
    }

    pub fn is_ready(&self) -> bool {
        match self {
            Self::Pending(..) => false,
            Self::Ready(..) => true,
        }
    }
}

#[derive(Debug)]
pub struct Environment {
    pub enclosing: Option<Arc<Environment>>,
    pub values: DashMap<u64, PackagedObject, ahash::RandomState>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            values: DashMap::with_hasher(ahash::RandomState::new()),
            enclosing: None,
        }
    }
}

impl Environment {
    pub fn new_with_parent(enclosing: Arc<Environment>) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: DashMap::with_hasher(ahash::RandomState::new()),
        }
    }

    pub fn get(&self, key: &u64) -> Option<Ref<'_, u64, PackagedObject, ahash::RandomState>> {
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

    pub fn get_at(
        &self,
        distance: usize,
        key: &u64,
    ) -> Option<Ref<'_, u64, PackagedObject, ahash::RandomState>> {
        self.ancestor(distance)?.values.get(key)
    }

    pub fn assign_at(&self, distance: usize, key: u64, value: LoxObject) -> Option<()> {
        self.ancestor(distance)?
            .values
            .insert(key, PackagedObject::Ready(Ok(value)));

        Some(())
    }

    pub fn ancestor(&self, distance: usize) -> Option<&Self> {
        let mut environment = self;

        for _ in 0..distance {
            environment = environment.enclosing.as_ref()?;
        }

        Some(environment)
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

pub fn put(environment: Arc<Environment>, locals: LocalsMap, name: &str, expr: Arc<Expression>) {
    let key = env_hash(name);

    // To avoid deadlock, we have to remove the old value
    let existing_key = environment.values.remove(&key);

    let condvar = Condvar::new();

    environment
        .values
        .insert(key, PackagedObject::Pending(Mutex::new(false), condvar));

    let sub_environment = create_sub_environment!(existing_key, environment);

    let executor = Executor {
        workers: &WORKERS,
        environment: Arc::clone(&sub_environment),
        locals: Arc::clone(&locals),
    };

    WORKERS.execute(move || {
        let value = executor.eval_expression(&expr);

        if let PackagedObject::Pending(mtx, cdv) = sub_environment.get(&key).unwrap().value() {
            *mtx.lock().unwrap() = true;
            cdv.notify_all();
        }

        environment.values.insert(key, PackagedObject::Ready(value));
    });
}

pub fn put_immediately(
    environment: Arc<Environment>,
    locals: LocalsMap,
    name: &str,
    expr_or_obj: Either<&Expression, LoxObject>,
) {
    let key = env_hash(name);
    // To avoid deadlock, we have to remove the old value
    let existing_key = environment.values.remove(&key);

    let sub_environment = create_sub_environment!(existing_key, environment);
    let sub_executor = Executor {
        environment: sub_environment,
        locals,
        workers: &WORKERS,
    };

    Arc::clone(&environment).values.insert(
        env_hash(name),
        PackagedObject::Ready(match expr_or_obj {
            Left(expr) => sub_executor.eval_expression(expr),
            Right(obj) => Ok(obj),
        }),
    );
}

pub fn env_hash(name: &str) -> u64 {
    let mut hasher = ahash::AHasher::default();
    hasher.write(name.as_bytes());
    hasher.finish()
}
