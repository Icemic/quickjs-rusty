#![allow(missing_docs)]

use std::{
    convert::TryFrom,
    ffi::{c_int, c_void},
    ptr::null_mut,
    sync::{Arc, Mutex},
};

use libquickjspp_sys as q;

use crate::callback::*;
use crate::console::ConsoleBackend;
use crate::errors::*;
use crate::module_loader::*;
use crate::utils::{create_string, ensure_no_excpetion, get_exception, make_cstring};
use crate::value::*;

use super::ContextBuilder;

/// Context is a wrapper around a QuickJS Javascript context.
/// It is the primary way to interact with the runtime.
///
/// For each `Context` instance a new instance of QuickJS
/// runtime is created. It means that it is safe to use
/// different contexts in different threads, but each
/// `Context` instance must be used only from a single thread.
pub struct Context {
    runtime: *mut q::JSRuntime,
    pub(crate) context: *mut q::JSContext,
    pub(crate) loop_context: Arc<Mutex<*mut q::JSContext>>,
    /// Stores callback closures and quickjs data pointers.
    /// This array is write-only and only exists to ensure the lifetime of
    /// the closure.
    // A Mutex is used over a RefCell because it needs to be unwind-safe.
    callbacks: Mutex<Vec<(Box<WrappedCallback>, Box<q::JSValue>)>>,
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            q::JS_FreeContext(self.context);
            q::JS_FreeRuntime(self.runtime);
        }
    }
}

impl Context {
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

    /// Initialize a wrapper by creating a JSRuntime and JSContext.
    pub fn new(memory_limit: Option<usize>) -> Result<Self, ContextError> {
        let runtime = unsafe { q::JS_NewRuntime() };
        if runtime.is_null() {
            return Err(ContextError::RuntimeCreationFailed);
        }

        // Configure memory limit if specified.
        if let Some(limit) = memory_limit {
            unsafe {
                q::JS_SetMemoryLimit(runtime, limit as _);
            }
        }

        let context = unsafe { q::JS_NewContext(runtime) };
        if context.is_null() {
            unsafe {
                q::JS_FreeRuntime(runtime);
            }
            return Err(ContextError::ContextCreationFailed);
        }

        // Initialize the promise resolver helper code.
        // This code is needed by Self::resolve_value
        let wrapper = Self {
            runtime,
            context,
            loop_context: Arc::new(Mutex::new(null_mut())),
            callbacks: Mutex::new(Vec::new()),
        };

        Ok(wrapper)
    }

    // See console standard: https://console.spec.whatwg.org
    pub fn set_console(&self, backend: Box<dyn ConsoleBackend>) -> Result<(), ExecutionError> {
        use crate::console::Level;

        self.add_callback("__console_write", move |args: Arguments| {
            let mut args = args.into_vec();

            if args.len() > 1 {
                let level_raw = args.remove(0);

                let level_opt = level_raw.to_string().ok().and_then(|v| match v.as_str() {
                    "trace" => Some(Level::Trace),
                    "debug" => Some(Level::Debug),
                    "log" => Some(Level::Log),
                    "info" => Some(Level::Info),
                    "warn" => Some(Level::Warn),
                    "error" => Some(Level::Error),
                    _ => None,
                });

                if let Some(level) = level_opt {
                    backend.log(level, args);
                }
            }
        })?;

        self.eval(
            r#"
            globalThis.console = {
                trace: (...args) => {
                    globalThis.__console_write("trace", ...args);
                },
                debug: (...args) => {
                    globalThis.__console_write("debug", ...args);
                },
                log: (...args) => {
                    globalThis.__console_write("log", ...args);
                },
                info: (...args) => {
                    globalThis.__console_write("info", ...args);
                },
                warn: (...args) => {
                    globalThis.__console_write("warn", ...args);
                },
                error: (...args) => {
                    globalThis.__console_write("error", ...args);
                },
            };
        "#,
        )?;

        Ok(())
    }

    /// Reset the Javascript engine.
    ///
    /// All state and callbacks will be removed.
    pub fn reset(self) -> Result<Self, ContextError> {
        unsafe {
            q::JS_FreeContext(self.context);
        };
        self.callbacks.lock().unwrap().clear();
        let context = unsafe { q::JS_NewContext(self.runtime) };
        if context.is_null() {
            return Err(ContextError::ContextCreationFailed);
        }

        let mut s = self;
        s.context = context;
        Ok(s)
    }

    /// Get raw pointer to the underlying QuickJS context.
    pub fn context_raw(&self) -> *mut q::JSContext {
        self.context
    }

    /// Get the global object.
    pub fn global(&self) -> Result<OwnedJsObject, ExecutionError> {
        let global_raw = unsafe { q::JS_GetGlobalObject(self.context) };
        let global_ref = OwnedJsValue::new(self.context, global_raw);
        let global = global_ref.try_into_object()?;
        Ok(global)
    }

    /// Set a global variable.
    ///
    /// ```rust
    /// use quickjspp::Context;
    /// let context = Context::builder().build().unwrap();
    ///
    /// context.set_global("someGlobalVariable", 42).unwrap();
    /// let value = context.eval_as::<i32>("someGlobalVariable").unwrap();
    /// assert_eq!(
    ///     value,
    ///     42,
    /// );
    /// ```
    pub fn set_global<T>(&self, name: &str, value: T) -> Result<(), ExecutionError>
    where
        T: ToOwnedJsValue,
    {
        let global = self.global()?;
        global.set_property(name, (self.context, value).into())?;
        Ok(())
    }

    /// Execute the pending job in the event loop.
    pub fn execute_pending_job(&self) -> Result<(), ExecutionError> {
        let ctx = &mut *self.loop_context.lock().unwrap();
        unsafe {
            loop {
                let err = q::JS_ExecutePendingJob(self.runtime, ctx as *mut _);

                if err <= 0 {
                    if err < 0 {
                        ensure_no_excpetion(*ctx)?
                    }
                    break;
                }
            }
        }
        Ok(())
    }

    /// If the given value is a promise, run the event loop until it is
    /// resolved, and return the final value.
    fn resolve_value(&self, value: OwnedJsValue) -> Result<OwnedJsValue, ExecutionError> {
        if value.is_exception() {
            let err = get_exception(self.context)
                .unwrap_or_else(|| ExecutionError::Internal("Unknown exception".to_string()));
            Err(err)
        } else if value.is_object() {
            let obj = value.try_into_object()?;
            if obj.is_promise()? {
                self.eval(
                    r#"
                    // Values:
                    //   - undefined: promise not finished
                    //   - false: error ocurred, __promiseError is set.
                    //   - true: finished, __promiseSuccess is set.
                    var __promiseResult = 0;
                    var __promiseValue = 0;

                    var __resolvePromise = function(p) {
                        p
                            .then(value => {
                                __promiseResult = true;
                                __promiseValue = value;
                            })
                            .catch(e => {
                                __promiseResult = false;
                                __promiseValue = e;
                            });
                    }
                "#,
                )?;

                let global = self.global()?;
                let resolver = global
                    .property_require("__resolvePromise")?
                    .try_into_function()?;

                // Call the resolver code that sets the result values once
                // the promise resolves.
                resolver.call(vec![obj.into_value()])?;

                loop {
                    let flag = unsafe {
                        let wrapper_mut = self as *const Self as *mut Self;
                        let ctx_mut = &mut (*wrapper_mut).context;
                        q::JS_ExecutePendingJob(self.runtime, ctx_mut)
                    };
                    if flag < 0 {
                        let e = get_exception(self.context).unwrap_or_else(|| {
                            ExecutionError::Internal("Unknown exception".to_string())
                        });
                        return Err(e);
                    }

                    // Check if promise is finished.
                    let res_val = global.property_require("__promiseResult")?;
                    if res_val.is_bool() {
                        let ok = res_val.to_bool()?;
                        let value = global.property_require("__promiseValue")?;

                        if ok {
                            return self.resolve_value(value);
                        } else {
                            let err_msg = value.js_to_string()?;
                            return Err(ExecutionError::Exception(OwnedJsValue::new(
                                self.context,
                                create_string(self.context, &err_msg).unwrap(),
                            )));
                        }
                    }
                }
            } else {
                Ok(obj.into_value())
            }
        } else {
            Ok(value)
        }
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
    /// use quickjspp::Context;
    /// let context = Context::builder().build().unwrap();
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
        let filename = "script.js";
        let filename_c = make_cstring(filename)?;
        let code_c = make_cstring(code)?;

        let value_raw = unsafe {
            q::JS_Eval(
                self.context,
                code_c.as_ptr(),
                code.len(),
                filename_c.as_ptr(),
                q::JS_EVAL_TYPE_GLOBAL as i32,
            )
        };
        let value = OwnedJsValue::new(self.context, value_raw);
        self.resolve_value(value)
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
    /// use quickjspp::Context;
    /// let context = Context::builder().build().unwrap();
    ///
    /// let value = context.eval_module("import {foo} from 'bar'; foo();");
    /// ```
    pub fn eval_module(&self, code: &str) -> Result<OwnedJsValue, ExecutionError> {
        let filename = "module.js";
        let filename_c = make_cstring(filename)?;
        let code_c = make_cstring(code)?;

        let value_raw = unsafe {
            q::JS_Eval(
                self.context,
                code_c.as_ptr(),
                code.len(),
                filename_c.as_ptr(),
                q::JS_EVAL_TYPE_MODULE as i32,
            )
        };
        let value = OwnedJsValue::new(self.context, value_raw);
        self.resolve_value(value)
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
    /// let context = Context::builder().build().unwrap();
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
        let value = self.eval(code)?;
        let ret = R::try_from(value).map_err(|e| e.into())?;
        Ok(ret)
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
    /// use quickjspp::Context;
    /// let context = Context::builder().build().unwrap();
    ///
    /// let value = context.run_module("./module");
    /// ```
    pub fn run_module(&self, filename: &str) -> Result<(), ExecutionError> {
        let filename_c = make_cstring(filename)?;

        unsafe {
            q::JS_RunModule(
                self.context,
                ".\0".as_ptr() as *const i8,
                filename_c.as_ptr(),
            )
        };

        ensure_no_excpetion(self.context)?;

        Ok(())
    }

    /// register module loader function, giving module name as input and return module code as output.
    pub fn set_module_loader(
        &self,
        module_loader_func: JSModuleLoaderFunc,
        module_normalize: Option<JSModuleNormalizeFunc>,
        opaque: *mut c_void,
    ) {
        let has_module_normalize = module_normalize.is_some();

        let module_loader = ModuleLoader {
            loader: module_loader_func,
            normalize: module_normalize,
            opaque,
        };

        let module_loader = Box::new(module_loader);
        let module_loader_ptr = Box::into_raw(module_loader);

        unsafe {
            if has_module_normalize {
                q::JS_SetModuleLoaderFunc(
                    self.runtime,
                    Some(js_module_normalize),
                    Some(js_module_loader),
                    module_loader_ptr as *mut c_void,
                );
            } else {
                q::JS_SetModuleLoaderFunc(
                    self.runtime,
                    None,
                    Some(js_module_loader),
                    module_loader_ptr as *mut c_void,
                );
            }
        }
    }

    /// Set the host promise rejection tracker.\
    /// This function works not as expected, see more details in the example.
    pub fn set_host_promise_rejection_tracker(
        &self,
        func: q::JSHostPromiseRejectionTracker,
        opaque: *mut c_void,
    ) {
        unsafe {
            q::JS_SetHostPromiseRejectionTracker(self.runtime, func, opaque);
        }
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
    /// use quickjspp::Context;
    /// let context = Context::builder().build().unwrap();
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
        args: impl IntoIterator<Item = impl ToOwnedJsValue>,
    ) -> Result<OwnedJsValue, ExecutionError> {
        let qargs = args
            .into_iter()
            .map(|v| (self.context, v).into())
            .collect::<Vec<OwnedJsValue>>();

        let global = self.global()?;
        let func = global
            .property_require(function_name)?
            .try_into_function()?;

        let ret = func.call(qargs)?;
        let v = self.resolve_value(ret)?;

        Ok(v)
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
    /// use quickjspp::{Context, OwnedJsValue};
    /// use std::collections::HashMap;
    ///
    /// let context = Context::builder().build().unwrap();
    ///
    /// // Register an object.
    /// let mut obj = HashMap::<String, OwnedJsValue>::new();
    /// let func = context
    ///         .create_callback(|a: i32, b: i32| a + b)
    ///         .unwrap();
    /// let func = OwnedJsValue::from((context.context_raw(), func));
    /// // insert add function into the object.
    /// obj.insert("add".to_string(), func);
    /// // insert the myObj to global.
    /// context.set_global("myObj", obj).unwrap();
    /// // Now we try out the 'myObj.add' function via eval.    
    /// let output = context.eval_as::<i32>("myObj.add( 3 , 4 ) ").unwrap();
    /// assert_eq!(output, 7);
    /// ```
    pub fn create_callback<'a, F>(
        &self,
        callback: impl Callback<F> + 'static,
    ) -> Result<JsFunction, ExecutionError> {
        let argcount = callback.argument_count() as i32;

        let context = self.context;
        let wrapper = move |argc: c_int, argv: *mut q::JSValue| -> q::JSValue {
            match exec_callback(context, argc, argv, &callback) {
                Ok(value) => value,
                // TODO: better error reporting.
                Err(e) => {
                    let js_exception_value = match e {
                        ExecutionError::Exception(e) => unsafe { e.extract() },
                        other => create_string(context, other.to_string().as_str()).unwrap(),
                    };
                    unsafe {
                        q::JS_Throw(context, js_exception_value);
                    }

                    unsafe { q::JS_NewSpecialValue(q::JS_TAG_EXCEPTION, 0) }
                }
            }
        };

        let (pair, trampoline) = unsafe { build_closure_trampoline(wrapper) };
        let data = (&*pair.1) as *const q::JSValue as *mut q::JSValue;
        self.callbacks.lock().unwrap().push(pair);

        let obj = unsafe {
            let f = q::JS_NewCFunctionData(self.context, trampoline, argcount, 0, 1, data);
            OwnedJsValue::new(self.context, f)
        };

        let f = obj.try_into_function()?;
        Ok(f)
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
    /// use quickjspp::Context;
    /// let context = Context::builder().build().unwrap();
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
    pub fn add_callback<'a, F>(
        &self,
        name: &str,
        callback: impl Callback<F> + 'static,
    ) -> Result<(), ExecutionError> {
        let cfunc = self.create_callback(callback)?;
        let global = self.global()?;
        global.set_property(name, cfunc.into_value())?;
        Ok(())
    }

    /// create a custom callback function
    pub fn create_custom_callback(
        &self,
        callback: CustomCallback,
    ) -> Result<JsFunction, ExecutionError> {
        let context = self.context;
        let wrapper = move |argc: c_int, argv: *mut q::JSValue| -> q::JSValue {
            let result = std::panic::catch_unwind(|| {
                let arg_slice = unsafe { std::slice::from_raw_parts(argv, argc as usize) };
                match callback(context, arg_slice) {
                    Ok(Some(value)) => value,
                    Ok(None) => unsafe { q::JS_NewSpecialValue(q::JS_TAG_UNDEFINED, 0) },
                    // TODO: better error reporting.
                    Err(e) => {
                        // TODO: should create an Error type.
                        let js_exception_value =
                            create_string(context, e.to_string().as_str()).unwrap();

                        unsafe {
                            q::JS_Throw(context, js_exception_value);
                        }

                        unsafe { q::JS_NewSpecialValue(q::JS_TAG_EXCEPTION, 0) }
                    }
                }
            });

            match result {
                Ok(v) => v,
                Err(_) => {
                    // TODO: should create an Error type.
                    let js_exception_value = create_string(context, "Callback panicked!").unwrap();

                    unsafe {
                        q::JS_Throw(context, js_exception_value);
                    }

                    unsafe { q::JS_NewSpecialValue(q::JS_TAG_EXCEPTION, 0) }
                }
            }
        };

        let (pair, trampoline) = unsafe { build_closure_trampoline(wrapper) };
        let data = (&*pair.1) as *const q::JSValue as *mut q::JSValue;
        self.callbacks.lock().unwrap().push(pair);

        let obj = unsafe {
            let f = q::JS_NewCFunctionData(self.context, trampoline, 0, 0, 1, data);
            OwnedJsValue::new(self.context, f)
        };

        let f = obj.try_into_function()?;
        Ok(f)
    }
}
