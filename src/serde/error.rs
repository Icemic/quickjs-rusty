use std::fmt::Display;

use serde::{de, ser};
use thiserror::Error as ThisError;

use crate::{ExecutionError, ValueError};

/// Error type for serde operations.
#[allow(missing_docs)]
#[derive(Debug, ThisError)]
pub enum Error {
    /// Error message.
    #[error("{0}")]
    Message(String),

    #[error("end of file")]
    Eof,
    #[error("invalid syntax")]
    Syntax,
    #[error("expect boolean")]
    ExpectedBoolean,
    #[error("expect integer")]
    ExpectedInteger,
    #[error("expect float")]
    ExpectedFloat,
    #[error("expect string")]
    ExpectedString,
    #[error("expect null")]
    ExpectedNull,
    #[error("expect object")]
    ExpectedObject,
    #[error("expect array or object")]
    ExpectedArrayOrObject,
    #[error("expect array")]
    ExpectedArray,
    #[error("expect array comma")]
    ExpectedArrayComma,
    #[error("expect array end")]
    ExpectedArrayEnd,
    #[error("expect map")]
    ExpectedMap,
    #[error("expect map colon")]
    ExpectedMapColon,
    #[error("expect map comma")]
    ExpectedMapComma,
    #[error("expect map end")]
    ExpectedMapEnd,
    #[error("expect enum")]
    ExpectedEnum,
    #[error("trailing characters")]
    TrailingCharacters,

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
