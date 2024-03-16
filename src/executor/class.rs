use std::fmt::Display;

use ahash::AHashMap;

use super::LoxCallable;

#[derive(Debug)]
pub struct LoxClass {
    pub name: String,
    pub methods: AHashMap<String, LoxCallable>,
}

impl LoxClass {
    pub fn new(name: String, methods: AHashMap<String, LoxCallable>) -> Self {
        Self { name, methods }
    }

    pub fn find_method(&self, method_name: &str) -> Option<&LoxCallable> {
        self.methods.get(method_name)
    }
}

impl PartialEq for LoxClass {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.methods.keys().eq(other.methods.keys())
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
