mod lox_error;

pub use lox_error::LoxError;

pub type LoxResult<'a, T> = Result<T, LoxError<'a>>;
