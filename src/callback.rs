use std::ffi::{c_int, c_void};
use std::{convert::TryFrom, marker::PhantomData, panic::RefUnwindSafe};

use anyhow::Result;
use libquickjs_ng_sys as q;

use crate::utils::create_string;
use crate::utils::create_undefined;
use crate::ExecutionError;
use crate::OwnedJsValue;
use crate::ValueError;

pub trait IntoCallbackResult {
    fn into_callback_res(self, context: *mut q::JSContext) -> Result<OwnedJsValue, String>;
}

impl<T> IntoCallbackResult for T
where
    OwnedJsValue: From<(*mut q::JSContext, T)>,
{
    fn into_callback_res(self, context: *mut q::JSContext) -> Result<OwnedJsValue, String> {
        Ok((context, self).into())
    }
}

impl<T, E: std::fmt::Display> IntoCallbackResult for Result<T, E>
where
    OwnedJsValue: From<(*mut q::JSContext, T)>,
{
    fn into_callback_res(self, context: *mut q::JSContext) -> Result<OwnedJsValue, String> {
        match self {
            Ok(v) => Ok((context, v).into()),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// The Callback trait is implemented for functions/closures that can be
/// used as callbacks in the JS runtime.
pub trait Callback<F>: RefUnwindSafe {
    /// Returns the number of required Javascript arguments.
    fn argument_count(&self) -> usize;

    /// Execute the callback.
    ///
    /// Should return:
    ///   - Err(_) if the JS values could not be converted
    ///   - Ok(Err(_)) if an error ocurred while processing.
    ///       The given error will be raised as a JS exception.
    ///   - Ok(Ok(result)) when execution succeeded.
    fn call(
        &self,
        context: *mut q::JSContext,
        args: Vec<OwnedJsValue>,
    ) -> Result<Result<OwnedJsValue, String>, ValueError>;
}

macro_rules! impl_callback {
    (@call $len:literal $self:ident $args:ident ) => {
        $self()
    };

    (@call $len:literal $self:ident $args:ident $( $arg:ident ),* ) => {
        {
            let mut iter = $args.into_iter();
            $self(
                $(
                    $arg::try_from(iter.next().unwrap())?,
                )*
            )
        }
    };

    [ $(  $len:literal : ( $( $arg:ident, )* ), )* ] => {
        $(

            impl<
                $( $arg, )*
                E,
                R,
                F,
            > Callback<PhantomData<(
                $( &$arg, )*
                &E,
                &R,
                &F,
            )>> for F
            where
                $( $arg: TryFrom<OwnedJsValue, Error = E>, )*
                ValueError: From<E>,
                R: IntoCallbackResult,
                F: Fn( $( $arg, )*  ) -> R + Sized + RefUnwindSafe,
            {
                fn argument_count(&self) -> usize {
                    $len
                }

                fn call(&self, context: *mut q::JSContext, args: Vec<OwnedJsValue>)
                    -> Result<Result<OwnedJsValue, String>, ValueError> {
                    if args.len() != $len {
                        return Ok(Err(format!(
                            "Invalid argument count: Expected {}, got {}",
                            self.argument_count(),
                            args.len()
                        )));
                    }

                    let res = impl_callback!(@call $len self args $($arg),* );
                    Ok(res.into_callback_res(context))
                }
            }
        )*
    };
}

impl<R, F> Callback<PhantomData<(&R, &F)>> for F
where
    R: IntoCallbackResult,
    F: Fn() -> R + Sized + RefUnwindSafe,
{
    fn argument_count(&self) -> usize {
        0
    }

    fn call(
        &self,
        context: *mut q::JSContext,
        args: Vec<OwnedJsValue>,
    ) -> Result<Result<OwnedJsValue, String>, ValueError> {
        if !args.is_empty() {
            return Ok(Err(format!(
                "Invalid argument count: Expected 0, got {}",
                args.len(),
            )));
        }

        let res = self();
        Ok(res.into_callback_res(context))
    }
}

impl_callback![
    1: (A1,),
    2: (A1, A2,),
    3: (A1, A2, A3,),
    4: (A1, A2, A3, A4,),
    5: (A1, A2, A3, A4, A5,),
];

/// A wrapper around Vec<JsValue>, used for vararg callbacks.
///
/// To create a callback with a variable number of arguments, a callback closure
/// must take a single `Arguments` argument.
pub struct Arguments(Vec<OwnedJsValue>);

impl Arguments {
    /// Unpack the arguments into a Vec.
    pub fn into_vec(self) -> Vec<OwnedJsValue> {
        self.0
    }
}

impl<F> Callback<PhantomData<(&Arguments, &F)>> for F
where
    F: Fn(Arguments) + Sized + RefUnwindSafe,
{
    fn argument_count(&self) -> usize {
        0
    }

    fn call(
        &self,
        context: *mut q::JSContext,
        args: Vec<OwnedJsValue>,
    ) -> Result<Result<OwnedJsValue, String>, ValueError> {
        (self)(Arguments(args));
        Ok(Ok(OwnedJsValue::new(context, create_undefined())))
    }
}

impl<F, R> Callback<PhantomData<(&Arguments, &F, &R)>> for F
where
    R: IntoCallbackResult,
    F: Fn(Arguments) -> R + Sized + RefUnwindSafe,
{
    fn argument_count(&self) -> usize {
        0
    }

    fn call(
        &self,
        context: *mut q::JSContext,
        args: Vec<OwnedJsValue>,
    ) -> Result<Result<OwnedJsValue, String>, ValueError> {
        let res = (self)(Arguments(args));
        Ok(res.into_callback_res(context))
    }
}

/// Helper for executing a callback closure.
pub fn exec_callback<F>(
    context: *mut q::JSContext,
    argc: c_int,
    argv: *mut q::JSValue,
    callback: &impl Callback<F>,
) -> Result<q::JSValue, ExecutionError> {
    let result = std::panic::catch_unwind(|| {
        let arg_slice = unsafe { std::slice::from_raw_parts(argv, argc as usize) };

        let args = arg_slice
            .iter()
            .map(|raw| OwnedJsValue::own(context, raw))
            .collect::<Vec<_>>();

        match callback.call(context, args) {
            Ok(Ok(result)) => {
                let serialized = unsafe { result.extract() };
                Ok(serialized)
            }
            // TODO: better error reporting.
            Ok(Err(e)) => Err(ExecutionError::Exception(OwnedJsValue::new(
                context,
                create_string(context, &e).unwrap(),
            ))),
            Err(e) => Err(e.into()),
        }
    });

    match result {
        Ok(r) => r,
        Err(_e) => Err(ExecutionError::Internal("Callback panicked!".to_string())),
    }
}

pub type CustomCallback = fn(*mut q::JSContext, &[q::JSValue]) -> Result<Option<q::JSValue>>;
pub type WrappedCallback = dyn Fn(c_int, *mut q::JSValue) -> q::JSValue;

/// Taken from: https://s3.amazonaws.com/temp.michaelfbryan.com/callbacks/index.html
///
/// Create a C wrapper function for a Rust closure to enable using it as a
/// callback function in the Quickjs runtime.
///
/// Both the boxed closure and the boxed data are returned and must be stored
/// by the caller to guarantee they stay alive.
pub unsafe fn build_closure_trampoline<F>(
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
        let closure_ptr = q::JS_Ext_GetPtr(*data);
        let closure: &mut F = &mut *(closure_ptr as *mut F);
        (*closure)(argc, argv)
    }

    let boxed_f = Box::new(closure);

    let data = Box::new(q::JS_Ext_NewPointer(
        q::JS_TAG_NULL,
        (&*boxed_f) as *const F as *mut c_void,
    ));

    ((boxed_f, data), Some(trampoline::<F>))
}
