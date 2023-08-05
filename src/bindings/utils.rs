use libquickjspp_sys as q;

use crate::{ExecutionError, JsValue, ValueError};

use super::{convert, OwnedJsValue};

#[inline]
pub fn serialize_value(
    context: *mut q::JSContext,
    value: JsValue,
) -> Result<OwnedJsValue, ExecutionError> {
    let serialized = convert::serialize_value(context, value)?;
    Ok(OwnedJsValue::new(context, serialized))
}

// Deserialize a quickjs runtime value into a Rust value.
#[inline]
pub(crate) fn to_value(
    context: *mut q::JSContext,
    value: &q::JSValue,
) -> Result<JsValue, ValueError> {
    convert::deserialize_value(context, value)
}

#[inline]
pub fn serialize_raw(
    context: *mut q::JSContext,
    value: JsValue,
) -> Result<q::JSValue, ExecutionError> {
    convert::serialize_value(context, value).map_err(|e| e.into())
}

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
                    Some(ExecutionError::Exception(JsValue::String(strval)))
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
