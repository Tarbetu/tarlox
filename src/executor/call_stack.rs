use dashmap::DashMap;
use std::sync::Arc;

use crate::LoxResult;

use super::{Executor, LoxCallable, LoxObject};

struct CallState {
    queue: Vec<Arc<Vec<LoxObject>>>,
    result: Option<LoxResult<LoxObject>>,
}

impl CallState {
    fn new_with_args(args: Arc<Vec<LoxObject>>) -> Self {
        Self {
            queue: vec![args],
            result: None,
        }
    }
}

pub struct CallStack {
    calls: DashMap<Arc<LoxCallable>, CallState, ahash::RandomState>,
}

impl CallStack {
    pub fn new() -> Self {
        Self {
            calls: DashMap::with_hasher(ahash::RandomState::new()),
        }
    }

    pub fn add_to_queue(&self, callable: Arc<LoxCallable>, args: Arc<Vec<LoxObject>>) {
        let call_state = self.calls.get_mut(&callable);

        if let Some(mut call_state) = call_state {
            call_state.value_mut().queue.push(args)
        } else {
            self.calls.insert(callable, CallState::new_with_args(args));
        }
    }

    pub fn run_queue(&self, callable: &LoxCallable, executor: &Executor) {
        let call_state = self.calls.get_mut(callable);

        if let Some(mut call_state) = call_state {
            let mut result = None;

            while let Some(args) = call_state.queue.pop() {
                result = Some(callable.call(executor, &args));

                if result.as_ref().is_some_and(|res| res.is_err()) {
                    break;
                }
            }

            call_state.result = result;
        }
    }

    pub fn get_result(&self, callable: &LoxCallable) -> Option<LoxResult<LoxObject>> {
        if let Some(call_state) = self.calls.get(callable) {
            call_state.result.as_ref().map(|res| match res {
                Ok(obj) => Ok(LoxObject::from(obj)),
                Err(e) => Err(e.to_owned()),
            })
        } else {
            None
        }
    }
}
