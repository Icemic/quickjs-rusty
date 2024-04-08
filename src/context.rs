mod builder;
mod wrapper;

pub use builder::ContextBuilder;
pub use wrapper::ContextWrapper;

use crate::*;

/// Context is a wrapper around a QuickJS Javascript context.
/// It is the primary way to interact with the runtime.
///
/// For each `Context` instance a new instance of QuickJS
/// runtime is created. It means that it is safe to use
/// different contexts in different threads, but each
/// `Context` instance must be used only from a single thread.
pub struct Context {
    wrapper: ContextWrapper,
}

impl Context {
    fn from_wrapper(wrapper: ContextWrapper) -> Self {
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
        let wrapper = ContextWrapper::new(None)?;
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
