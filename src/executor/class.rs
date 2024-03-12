use std::{fmt::Display, sync::Arc};

#[derive(Debug, PartialEq, Clone)]
pub struct LoxClass {
    pub name: Arc<String>,
}

impl LoxClass {
    pub fn new(name: String) -> Self {
        Self {
            name: Arc::new(name),
        }
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
