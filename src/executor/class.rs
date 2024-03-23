use std::{fmt::Display, sync::Arc};

use ahash::AHashMap;

use super::LoxCallable;

#[derive(Debug)]
pub struct LoxClass {
    pub name: String,
    pub superclass: Option<Arc<Self>>,
    pub methods: AHashMap<String, LoxCallable>,
}

impl LoxClass {
    pub fn new(
        name: String,
        superclass: Option<Arc<Self>>,
        methods: AHashMap<String, LoxCallable>,
    ) -> Self {
        Self {
            name,
            superclass,
            methods,
        }
    }

    pub fn find_method(&self, method_name: &str) -> Option<&LoxCallable> {
        let mut result = self.methods.get(method_name);

        if let (None, Some(superclass)) = (result, self.superclass.as_ref()) {
            result = superclass.find_method(method_name);
        }

        result
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
