use std::fmt::Display;

use serde::{de, ser};
use thiserror::Error as ThisError;

use crate::{ExecutionError, ValueError};

/// Error type for serde operations.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Error message.
    #[error("{0}")]
    Message(String),

    /// transparent enum value for [ValueError].
    #[error(transparent)]
    ValueError(#[from] ValueError),
    /// transparent enum value for [ExecutionError].
    #[error(transparent)]
    ExecutionError(#[from] ExecutionError),
    /// transparent enum value for [std::io::Error].
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    /// transparent enum value for [anyhow::Error].
    #[error(transparent)]
    Others(#[from] anyhow::Error),
}

/// Result type for serde operations.
pub type Result<T> = anyhow::Result<T, Error>;

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
