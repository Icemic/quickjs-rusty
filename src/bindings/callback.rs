use std::os::raw::c_int;

use anyhow::Result;
use libquickjspp_sys as q;

use crate::utils::create_string;
use crate::OwnedJsValue;
use crate::{callback::Callback, ExecutionError};

/// Helper for executing a callback closure.
pub(super) fn exec_callback<F>(
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
