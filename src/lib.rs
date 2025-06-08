//! quickjs-rusty is a a Rust wrapper for [QuickJS-NG](https://github.com/quickjs-ng/quickjs), a new Javascript
//! engine by Fabrice Bellard.
//!
//! It enables easy and straight-forward execution of modern Javascript from Rust.
//!
//! ## Quickstart:
//!
//! ```rust
//! use quickjs_rusty::Context;
//!
//! let context = Context::builder().build().unwrap();
//!
//! // Eval.
//!
//! let value = context.eval("1 + 2", false).unwrap();
//! assert_eq!(value.to_int(), Ok(3));
//!
//! let value = context.eval_as::<String>(" var x = 100 + 250; x.toString() ").unwrap();
//! assert_eq!(&value, "350");
//!
//! // Callbacks.
//!
//! context.add_callback("myCallback", |a: i32, b: i32| a + b).unwrap();
//!
//! context.eval(r#"
//!     // x will equal 30
//!     var x = myCallback(10, 20);
//! "#, false).unwrap();
//! ```

// #![deny(missing_docs)]

mod callback;
pub mod compile;
pub mod console;
pub mod context;
pub mod errors;
pub mod module_loader;
#[cfg(feature = "serde")]
pub mod serde;
pub mod utils;
pub mod value;

pub use libquickjs_ng_sys::{JSContext, JSValue as RawJSValue};

pub use self::callback::{Arguments, Callback};
pub use self::context::*;
pub use self::errors::*;
pub use self::value::*;
