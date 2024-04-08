use std::{error, fmt};

use super::ExecutionError;

/// Error on context creation.
#[derive(Debug)]
pub enum ContextError {
    /// Runtime could not be created.
    RuntimeCreationFailed,
    /// Context could not be created.
    ContextCreationFailed,
    /// Execution error while building.
    Execution(ExecutionError),
    #[doc(hidden)]
    __NonExhaustive,
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ContextError::*;
        match self {
            RuntimeCreationFailed => write!(f, "Could not create runtime"),
            ContextCreationFailed => write!(f, "Could not create context"),
            Execution(e) => e.fmt(f),
            __NonExhaustive => unreachable!(),
        }
    }
}

impl error::Error for ContextError {}
