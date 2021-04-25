use rustyline;
use std::io;

pub type Error = Box<dyn std::error::Error>;

#[derive(Debug)]
pub struct StringError(pub String);
impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {}", self.0)
    }
}
impl std::error::Error for StringError {}

pub type Result<T> = std::result::Result<T, Error>;

// build a string error
#[macro_export]
macro_rules! se {
    ($($arg:tt)*) => {{ crate::errors::StringError(format!($($arg)*)) }};
}
