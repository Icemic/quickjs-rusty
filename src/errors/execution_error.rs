use std::{error, fmt};

use crate::OwnedJsValue;

use super::ValueError;

/// Error on Javascript execution.
#[derive(Debug)]
pub enum ExecutionError {
    /// Code to be executed contained zero-bytes.
    InputWithZeroBytes,
    /// Value conversion failed. (either input arguments or result value).
    Conversion(ValueError),
    /// Internal error.
    Internal(String),
    /// JS Exception was thrown.
    Exception(OwnedJsValue),
    /// JS Runtime exceeded the memory limit.
    OutOfMemory,
    #[doc(hidden)]
    __NonExhaustive,
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ExecutionError::*;
        match self {
            InputWithZeroBytes => write!(f, "Invalid script input: code contains zero byte (\\0)"),
            Conversion(e) => e.fmt(f),
            Internal(e) => write!(f, "Internal error: {}", e),
            Exception(e) => {
                if e.is_string() {
                    write!(f, "{}", e.to_string().unwrap())
                } else {
                    write!(f, "JS Exception: {:?}", e)
                }
            }
            OutOfMemory => write!(f, "Out of memory: runtime memory limit exceeded"),
            __NonExhaustive => unreachable!(),
        }
    }
}

impl PartialEq for ExecutionError {
    fn eq(&self, other: &Self) -> bool {
        let left = self.to_string();
        let right = other.to_string();
        left == right
    }
}

impl error::Error for ExecutionError {}

impl From<ValueError> for ExecutionError {
    fn from(v: ValueError) -> Self {
        ExecutionError::Conversion(v)
    }
}
