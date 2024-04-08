//! quickjspp is a a Rust wrapper for [QuickJSpp](https://github.com/c-smile/quickjspp), a new Javascript
//! engine by Fabrice Bellard.
//!
//! It enables easy and straight-forward execution of modern Javascript from Rust.
//!
//! ## Quickstart:
//!
//! ```rust
//! use quickjspp::{Context, JsValue};
//!
//! let context = Context::new().unwrap();
//!
//! // Eval.
//!
//! let value = context.eval("1 + 2").unwrap();
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
//! "#).unwrap();
//! ```

// #![deny(missing_docs)]

#[cfg(feature = "bigint")]
pub(crate) mod bigint;
mod bindings;
mod callback;
pub mod console;
pub mod errors;
#[cfg(feature = "serde")]
pub mod serde;
pub mod utils;

use std::{convert::TryFrom, error, ffi::c_void, fmt};

use libquickjspp_sys::JSHostPromiseRejectionTracker;
pub use libquickjspp_sys::{JSContext, JSValue as RawJSValue};

pub use self::{
    bindings::*,
    callback::{Arguments, Callback},
    errors::ValueError,
};
#[cfg(feature = "bigint")]
pub use bigint::BigInt;

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
                    write!(f, "{:?}", e.to_string().unwrap())
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

/// A builder for [Context](Context).
///
/// Create with [Context::builder](Context::builder).
pub struct ContextBuilder {
    memory_limit: Option<usize>,
    console_backend: Option<Box<dyn console::ConsoleBackend>>,
}

impl ContextBuilder {
    fn new() -> Self {
        Self {
            memory_limit: None,
            console_backend: None,
        }
    }

    /// Sets the memory limit of the Javascript runtime (in bytes).
    ///
    /// If the limit is exceeded, methods like `eval` will return
    /// a `Err(ExecutionError::Exception(JsValue::Null))`
    // TODO: investigate why we don't get a proper exception message here.
    pub fn memory_limit(self, max_bytes: usize) -> Self {
        let mut s = self;
        s.memory_limit = Some(max_bytes);
        s
    }

    /// Set a console handler that will proxy `console.{log,trace,debug,...}`
    /// calls.
    ///
    /// The given argument must implement the [console::ConsoleBackend] trait.
    ///
    /// A very simple logger could look like this:
    pub fn console<B>(mut self, backend: B) -> Self
    where
        B: console::ConsoleBackend,
    {
        self.console_backend = Some(Box::new(backend));
        self
    }

    /// Finalize the builder and build a JS Context.
    pub fn build(self) -> Result<Context, ContextError> {
        let wrapper = bindings::ContextWrapper::new(self.memory_limit)?;
        if let Some(be) = self.console_backend {
            wrapper.set_console(be).map_err(ContextError::Execution)?;
        }
        Ok(Context::from_wrapper(wrapper))
    }
}

/// Context is a wrapper around a QuickJS Javascript context.
/// It is the primary way to interact with the runtime.
///
/// For each `Context` instance a new instance of QuickJS
/// runtime is created. It means that it is safe to use
/// different contexts in different threads, but each
/// `Context` instance must be used only from a single thread.
pub struct Context {
    wrapper: bindings::ContextWrapper,
}

impl Context {
    fn from_wrapper(wrapper: bindings::ContextWrapper) -> Self {
        Self { wrapper }
    }

    /// Create a `ContextBuilder` that allows customization of JS Runtime settings.
    ///
    /// For details, see the methods on `ContextBuilder`.
    ///
    /// ```rust
    /// let _context = quickjspp::Context::builder()
    ///     .memory_limit(100_000)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> ContextBuilder {
        ContextBuilder::new()
    }

    /// Create a new Javascript context with default settings.
    pub fn new() -> Result<Self, ContextError> {
        let wrapper = bindings::ContextWrapper::new(None)?;
        Ok(Self::from_wrapper(wrapper))
    }

    /// Reset the Javascript engine.
    ///
    /// All state and callbacks will be removed.
    pub fn reset(self) -> Result<Self, ContextError> {
        let wrapper = self.wrapper.reset()?;
        Ok(Self { wrapper })
    }

    /// Get raw pointer to the underlying QuickJS context.
    pub fn context_raw(&self) -> *mut JSContext {
        self.wrapper.context
    }

    /// Evaluates Javascript code and returns the value of the final expression.
    ///
    /// **Promises**:
    /// If the evaluated code returns a Promise, the event loop
    /// will be executed until the promise is finished. The final value of
    /// the promise will be returned, or a `ExecutionError::Exception` if the
    /// promise failed.
    ///
    /// ```rust
    /// use quickjspp::{Context, JsValue};
    /// let context = Context::new().unwrap();
    ///
    /// let value = context.eval(" 1 + 2 + 3 ").unwrap();
    /// assert_eq!(
    ///     value.to_int(),
    ///     Ok(6),
    /// );
    ///
    /// let value = context.eval(r#"
    ///     function f() { return 55 * 3; }
    ///     let y = f();
    ///     var x = y.toString() + "!"
    ///     x
    /// "#).unwrap();
    /// assert_eq!(
    ///     value.to_string().unwrap(),
    ///     "165!",
    /// );
    /// ```
    pub fn eval(&self, code: &str) -> Result<OwnedJsValue, ExecutionError> {
        let value = self.wrapper.eval(code)?;
        Ok(value)
    }

    /// Evaluates Javascript code and returns the value of the final expression
    /// on module mode.
    ///
    /// **Promises**:
    /// If the evaluated code returns a Promise, the event loop
    /// will be executed until the promise is finished. The final value of
    /// the promise will be returned, or a `ExecutionError::Exception` if the
    /// promise failed.
    ///
    /// **Returns**:
    /// Return value will always be undefined on module mode.
    ///
    /// ```ignore
    /// use quickjspp::{Context, JsValue};
    /// let context = Context::new().unwrap();
    ///
    /// let value = context.eval_module("import {foo} from 'bar'; foo();");
    /// ```
    pub fn eval_module(&self, code: &str) -> Result<OwnedJsValue, ExecutionError> {
        let value = self.wrapper.eval_module(code)?;
        Ok(value)
    }

    /// Evaluates Javascript code and returns the value of the final expression
    /// on module mode.
    ///
    /// **Promises**:
    /// If the evaluated code returns a Promise, the event loop
    /// will be executed until the promise is finished. The final value of
    /// the promise will be returned, or a `ExecutionError::Exception` if the
    /// promise failed.
    ///
    /// ```ignore
    /// use quickjspp::{Context, JsValue};
    /// let context = Context::new().unwrap();
    ///
    /// let value = context.run_module("./module");
    /// ```
    pub fn run_module(&self, module_name: &str) -> Result<(), ExecutionError> {
        self.wrapper.run_module(module_name)?;
        Ok(())
    }

    /// register module loader function, giving module name as input and return module code as output.
    pub fn set_module_loader(
        &self,
        loader: JSModuleLoaderFunc,
        normalize: Option<JSModuleNormalizeFunc>,
        opaque: *mut c_void,
    ) {
        self.wrapper.set_module_loader(loader, normalize, opaque);
    }

    /// Set a promise rejection tracker.
    pub fn set_host_promise_rejection_tracker(
        &self,
        tracker: JSHostPromiseRejectionTracker,
        opaque: *mut c_void,
    ) {
        self.wrapper
            .set_host_promise_rejection_tracker(tracker, opaque);
    }

    /// Evaluates Javascript code and returns the value of the final expression
    /// as a Rust type.
    ///
    /// **Promises**:
    /// If the evaluated code returns a Promise, the event loop
    /// will be executed until the promise is finished. The final value of
    /// the promise will be returned, or a `ExecutionError::Exception` if the
    /// promise failed.
    ///
    /// ```rust
    /// use quickjspp::{Context};
    /// let context = Context::new().unwrap();
    ///
    /// let res = context.eval_as::<bool>(" 100 > 10 ");
    /// assert_eq!(
    ///     res,
    ///     Ok(true),
    /// );
    ///
    /// let value: i32 = context.eval_as(" 10 + 10 ").unwrap();
    /// assert_eq!(
    ///     value,
    ///     20,
    /// );
    /// ```
    pub fn eval_as<R>(&self, code: &str) -> Result<R, ExecutionError>
    where
        R: TryFrom<OwnedJsValue>,
        R::Error: Into<ValueError>,
    {
        let value = self.wrapper.eval(code)?;
        let ret = R::try_from(value).map_err(|e| e.into())?;
        Ok(ret)
    }

    /// Set a global variable.
    ///
    /// ```rust
    /// use quickjspp::{Context, JsValue};
    /// let context = Context::new().unwrap();
    ///
    /// context.set_global("someGlobalVariable", 42).unwrap();
    /// let value = context.eval_as::<i32>("someGlobalVariable").unwrap();
    /// assert_eq!(
    ///     value,
    ///     42,
    /// );
    /// ```
    pub fn set_global(&self, name: &str, value: OwnedJsValue) -> Result<(), ExecutionError> {
        let global = self.wrapper.global()?;
        global.set_property(name, value)?;
        Ok(())
    }

    /// Call a global function in the Javascript namespace.
    ///
    /// **Promises**:
    /// If the evaluated code returns a Promise, the event loop
    /// will be executed until the promise is finished. The final value of
    /// the promise will be returned, or a `ExecutionError::Exception` if the
    /// promise failed.
    ///
    /// ```rust
    /// use quickjspp::{Context, JsValue};
    /// let context = Context::new().unwrap();
    ///
    /// let res = context.call_function("encodeURIComponent", vec!["a=b"]).unwrap();
    /// assert_eq!(
    ///     res.to_string(),
    ///     Ok("a%3Db".to_string()),
    /// );
    /// ```
    pub fn call_function(
        &self,
        function_name: &str,
        args: impl IntoIterator<Item = OwnedJsValue>,
    ) -> Result<OwnedJsValue, ExecutionError> {
        let qargs = args.into_iter().collect::<Vec<OwnedJsValue>>();

        let global = self.wrapper.global()?;
        let func = global
            .property_require(function_name)?
            .try_into_function()?;
        let v = self.wrapper.call_function(func, qargs)?;
        Ok(v)
    }

    /// Add a global JS function that is backed by a Rust function or closure.
    ///
    /// The callback must satisfy several requirements:
    /// * accepts 0 - 5 arguments
    /// * each argument must be convertible from a JsValue
    /// * must return a value
    /// * the return value must either:
    ///   - be convertible to JsValue
    ///   - be a Result<T, E> where T is convertible to JsValue
    ///     if Err(e) is returned, a Javascript exception will be raised
    ///
    /// ```rust
    /// use quickjspp::{Context, JsValue};
    /// let context = Context::new().unwrap();
    ///
    /// // Register a closue as a callback under the "add" name.
    /// // The 'add' function can now be called from Javascript code.
    /// context.add_callback("add", |a: i32, b: i32| { a + b }).unwrap();
    ///
    /// // Now we try out the 'add' function via eval.
    /// let output = context.eval_as::<i32>(" add( 3 , 4 ) ").unwrap();
    /// assert_eq!(
    ///     output,
    ///     7,
    /// );
    /// ```
    pub fn add_callback<F>(
        &self,
        name: &str,
        callback: impl Callback<F> + 'static,
    ) -> Result<(), ExecutionError> {
        self.wrapper.add_callback(name, callback)
    }

    /// create a custom callback function
    pub fn create_custom_callback(
        &self,
        callback: CustomCallback,
    ) -> Result<JsFunction, ExecutionError> {
        self.wrapper.create_custom_callback(callback)
    }

    /// Create a JS function that is backed by a Rust function or closure.
    /// Can be used to create a function and add it to an object.
    ///
    /// The callback must satisfy several requirements:
    /// * accepts 0 - 5 arguments
    /// * each argument must be convertible from a JsValue
    /// * must return a value
    /// * the return value must either:
    ///   - be convertible to JsValue
    ///   - be a Result<T, E> where T is convertible to JsValue
    ///     if Err(e) is returned, a Javascript exception will be raised
    ///
    /// ```rust
    /// use quickjspp::{Context, JsValue};
    /// use std::collections::HashMap;
    ///
    /// let context = Context::new().unwrap();
    ///
    /// // Register an object.
    /// let mut obj = HashMap::<String, JsValue>::new();

    /// // insert add function into the object.
    /// obj.insert(
    ///     "add".to_string(),
    ///     context
    ///         .create_callback(|a: i32, b: i32| a + b)
    ///         .unwrap()
    ///         .into(),
    /// );
    /// // insert the myObj to global.
    /// context.set_global("myObj", obj).unwrap();
    /// // Now we try out the 'myObj.add' function via eval.    
    /// let output = context.eval_as::<i32>("myObj.add( 3 , 4 ) ").unwrap();
    /// assert_eq!(output, 7);
    /// ```
    pub fn create_callback<F>(
        &self,
        callback: impl Callback<F> + 'static,
    ) -> Result<JsFunction, ExecutionError> {
        self.wrapper.create_callback(callback)
    }

    ///
    pub fn execute_pending_job(&self) -> Result<(), ExecutionError> {
        self.wrapper.execute_pending_job()
    }
}
