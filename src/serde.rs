//! Serde integration.

mod de;
mod error;
mod ser;
mod utils;

pub use de::{from_js, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_js, Serializer};
