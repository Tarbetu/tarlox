mod lox_error;

pub use lox_error::LoxError;

pub type LoxResult<T> = Result<T, LoxError>;
