use libquickjspp_sys as q;

use crate::utils::create_string;
use crate::ExecutionError;

use super::OwnedJsValue;

/// Get the last exception from the runtime, and if present, convert it to a ExceptionError.
pub(crate) fn get_exception(context: *mut q::JSContext) -> Option<ExecutionError> {
    let value = unsafe {
        let raw = q::JS_GetException(context);
        OwnedJsValue::new(context, raw)
    };

    if value.is_null() {
        None
    } else if value.is_exception() {
        Some(ExecutionError::Internal(
            "Could get exception from runtime".into(),
        ))
    } else {
        match value.js_to_string() {
            Ok(strval) => {
                if strval.contains("out of memory") {
                    Some(ExecutionError::OutOfMemory)
                } else {
                    Some(ExecutionError::Exception(OwnedJsValue::new(
                        context,
                        create_string(context, &strval).unwrap(),
                    )))
                }
            }
            Err(e) => Some(e),
        }
    }
}

/// Returns `Result::Err` when an error ocurred.
pub(crate) fn ensure_no_excpetion(context: *mut q::JSContext) -> Result<(), ExecutionError> {
    if let Some(e) = get_exception(context) {
        Err(e)
    } else {
        Ok(())
    }
}
