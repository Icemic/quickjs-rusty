use std::{error, fmt};

/// Error during value conversion.
#[derive(PartialEq, Eq, Debug)]
pub enum ValueError {
    /// Invalid non-utf8 string.
    InvalidString(std::str::Utf8Error),
    /// Encountered string with \0 bytes.
    StringWithZeroBytes(std::ffi::NulError),
    /// Internal error.
    Internal(String),
    ///
    BigIntOverflow,
    /// Received an unexpected type that could not be converted.
    UnexpectedType,
    #[doc(hidden)]
    __NonExhaustive,
}

// TODO: remove this once either the Never type get's stabilized or the compiler
// can properly handle Infallible.
impl From<std::convert::Infallible> for ValueError {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ValueError::*;
        match self {
            InvalidString(e) => write!(
                f,
                "Value conversion failed - invalid non-utf8 string: {}",
                e
            ),
            StringWithZeroBytes(_) => write!(f, "String contains \\0 bytes",),
            Internal(e) => write!(f, "Value conversion failed - internal error: {}", e),
            BigIntOverflow => write!(f, "BigInt overflow"),
            UnexpectedType => write!(f, "Could not convert - received unexpected type"),
            __NonExhaustive => unreachable!(),
        }
    }
}

impl error::Error for ValueError {}
