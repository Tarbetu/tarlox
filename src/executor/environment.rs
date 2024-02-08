use ahash::AHashMap;
use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::{syntax::Expression, LoxResult};

use super::{eval_expression, object::LoxObject};

use std::{
    num::NonZeroUsize,
    sync::mpsc::{channel, Receiver},
    thread::available_parallelism,
};

enum PackagedObject {
    Pending(Receiver<LoxResult<LoxObject>>),
    Ready(LoxResult<LoxObject>),
}

pub struct Environment {
    values: AHashMap<String, PackagedObject>,
    pool: ThreadPool,
}

impl Environment {
    fn new() -> Self {
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

    fn put(&mut self, name: String, expr: Expression) {
        let (sender, receiver) = channel();
        self.values.insert(name, PackagedObject::Pending(receiver));

        self.pool.install(move || {
            sender.send(eval_expression(&expr)).unwrap();
        });
    }

    fn get(&mut self, name: String) -> Option<&PackagedObject> {
        use PackagedObject::*;

        let status = self.values.get_mut(&name)?;

        match status {
            Pending(recv) => match recv.try_recv() {
                Ok(obj) => {
                    *status = PackagedObject::Ready(obj);
                    Some(status)
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => Some(status),
                Err(_) => panic!("Receiver is disconnected!"),
            },
            Ready(_) => Some(status),
        }
    }
}
