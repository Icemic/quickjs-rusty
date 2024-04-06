#![allow(missing_docs)]

mod callback;
mod compile;
mod droppable_value;
mod module;
mod value;

use std::{
    ffi::CString,
    os::raw::{c_int, c_void},
    ptr::null_mut,
    sync::{Arc, Mutex},
};

use libquickjspp_sys as q;

use crate::{
    callback::{Arguments, Callback},
    console::ConsoleBackend,
    utils::{create_string, ensure_no_excpetion, get_exception},
    ContextError, ExecutionError, ValueError,
};

pub use value::*;

pub use self::callback::*;
use self::module::{js_module_loader, js_module_normalize, ModuleLoader};
pub use self::module::{JSModuleLoaderFunc, JSModuleNormalizeFunc};
pub use droppable_value::DroppableValue;

/// Helper for creating CStrings.
fn make_cstring(value: impl Into<Vec<u8>>) -> Result<CString, ValueError> {
    CString::new(value).map_err(ValueError::StringWithZeroBytes)
}

type WrappedCallback = dyn Fn(c_int, *mut q::JSValue) -> q::JSValue;

/// Taken from: https://s3.amazonaws.com/temp.michaelfbryan.com/callbacks/index.html
///
/// Create a C wrapper function for a Rust closure to enable using it as a
/// callback function in the Quickjs runtime.
///
/// Both the boxed closure and the boxed data are returned and must be stored
/// by the caller to guarantee they stay alive.
unsafe fn build_closure_trampoline<F>(
    closure: F,
) -> ((Box<WrappedCallback>, Box<q::JSValue>), q::JSCFunctionData)
where
    F: Fn(c_int, *mut q::JSValue) -> q::JSValue + 'static,
{
    unsafe extern "C" fn trampoline<F>(
        _ctx: *mut q::JSContext,
        _this: q::JSValue,
        argc: c_int,
        argv: *mut q::JSValue,
        _magic: c_int,
        data: *mut q::JSValue,
    ) -> q::JSValue
    where
        F: Fn(c_int, *mut q::JSValue) -> q::JSValue,
    {
        let closure_ptr = q::JS_VALUE_GET_PTR(*data);
        let closure: &mut F = &mut *(closure_ptr as *mut F);
        (*closure)(argc, argv)
    }

    let boxed_f = Box::new(closure);

    let data = Box::new(q::JS_NewPointer(
        q::JS_TAG_NULL,
        (&*boxed_f) as *const F as *mut c_void,
    ));

    ((boxed_f, data), Some(trampoline::<F>))
}

/*
type ModuleInit = dyn Fn(*mut q::JSContext, *mut q::JSModuleDef);

thread_local! {
    static NATIVE_MODULE_INIT: RefCell<Option<Box<ModuleInit>>> = RefCell::new(None);
}

unsafe extern "C" fn native_module_init(
    ctx: *mut q::JSContext,
    m: *mut q::JSModuleDef,
) -> ::std::os::raw::c_int {
    NATIVE_MODULE_INIT.with(|init| {
        let init = init.replace(None).unwrap();
        init(ctx, m);
    });
    0
}
*/

/// Wraps a quickjs context.
///
/// Cleanup of the context happens in drop.
pub struct ContextWrapper {
    runtime: *mut q::JSRuntime,
    pub(crate) context: *mut q::JSContext,
    pub(crate) loop_context: Arc<Mutex<*mut q::JSContext>>,
    /// Stores callback closures and quickjs data pointers.
    /// This array is write-only and only exists to ensure the lifetime of
    /// the closure.
    // A Mutex is used over a RefCell because it needs to be unwind-safe.
    callbacks: Mutex<Vec<(Box<WrappedCallback>, Box<q::JSValue>)>>,
}

impl Drop for ContextWrapper {
    fn drop(&mut self) {
        unsafe {
            q::JS_FreeContext(self.context);
            q::JS_FreeRuntime(self.runtime);
        }
    }
}

impl ContextWrapper {
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

    /// Reset the wrapper by creating a new context.
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

    /// Get the global object.
    pub fn global(&self) -> Result<OwnedJsObject, ExecutionError> {
        let global_raw = unsafe { q::JS_GetGlobalObject(self.context) };
        let global_ref = OwnedJsValue::new(self.context, global_raw);
        let global = global_ref.try_into_object()?;
        Ok(global)
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

    /// Evaluate javascript code.
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

    /// Evaluate javascript module code.
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

    /// Evaluate Javascript module
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

    /*
    /// Call a constructor function.
    fn call_constructor(
        & self,
        function: OwnedJsValue,
        args: Vec<OwnedJsValue>,
    ) -> Result<OwnedJsValue, ExecutionError> {
        let mut qargs = args.iter().map(|arg| arg.value).collect::<Vec<_>>();

        let value_raw = unsafe {
            q::JS_CallConstructor(
                self.context,
                function.value,
                qargs.len() as i32,
                qargs.as_mut_ptr(),
            )
        };
        let value = OwnedJsValue::new(self, value_raw);
        if value.is_exception() {
            let err = self
                .get_exception()
                .unwrap_or_else(|| ExecutionError::Exception("Unknown exception".into()));
            Err(err)
        } else {
            Ok(value)
        }
    }
    */

    /// Call a JS function with the given arguments.
    pub fn call_function(
        &self,
        function: JsFunction,
        args: Vec<OwnedJsValue>,
    ) -> Result<OwnedJsValue, ExecutionError> {
        let ret = function.call(args)?;
        self.resolve_value(ret)
    }

    /// Create a wrapped callback function.
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

    /// Create a raw callback function
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
