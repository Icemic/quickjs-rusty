//! utils

use std::ffi::CString;

use libquickjs_ng_sys as q;

use crate::value::{JsFunction, OwnedJsValue};
use crate::ValueError;

pub(crate) fn deserialize_borrowed_str(
    context: *mut q::JSContext,
    value: &q::JSValue,
) -> Result<&str, ValueError> {
    let r = value;
    let tag = unsafe { q::JS_Ext_ValueGetTag(*r) };

    match tag {
        q::JS_TAG_STRING => {
            let ptr = unsafe { q::JS_ToCStringLen2(context, std::ptr::null_mut(), *r, false) };

            if ptr.is_null() {
                return Err(ValueError::Internal(
                    "Could not convert string: got a null pointer".into(),
                ));
            }

            let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };

            let s = cstr
                .to_str()
                .map_err(ValueError::InvalidString)?
                .to_string()
                .leak();

            // Free the c string.
            unsafe { q::JS_FreeCString(context, ptr) };

            Ok(s)
        }
        _ => Err(ValueError::Internal(format!(
            "Expected a string, got a {:?}",
            tag
        ))),
    }
}

/// Helper for creating CStrings.
#[inline]
pub fn make_cstring(value: impl Into<Vec<u8>>) -> Result<CString, ValueError> {
    CString::new(value).map_err(ValueError::StringWithZeroBytes)
}

#[cfg(feature = "chrono")]
pub fn js_date_constructor(context: *mut q::JSContext) -> q::JSValue {
    let global = unsafe { q::JS_GetGlobalObject(context) };
    let tag = unsafe { q::JS_Ext_ValueGetTag(global) };
    assert_eq!(tag, q::JS_TAG_OBJECT);

    let date_constructor = unsafe {
        q::JS_GetPropertyStr(
            context,
            global,
            std::ffi::CStr::from_bytes_with_nul(b"Date\0")
                .unwrap()
                .as_ptr(),
        )
    };
    let tag = unsafe { q::JS_Ext_ValueGetTag(date_constructor) };
    assert_eq!(tag, q::JS_TAG_OBJECT);
    unsafe { q::JS_FreeValue(context, global) };
    date_constructor
}

#[cfg(feature = "bigint")]
fn js_create_bigint_function(context: *mut q::JSContext) -> q::JSValue {
    let global = unsafe { q::JS_GetGlobalObject(context) };
    let tag = unsafe { q::JS_Ext_ValueGetTag(global) };
    assert_eq!(tag, q::JS_TAG_OBJECT);

    let bigint_function = unsafe {
        q::JS_GetPropertyStr(
            context,
            global,
            std::ffi::CStr::from_bytes_with_nul(b"BigInt\0")
                .unwrap()
                .as_ptr(),
        )
    };
    let tag = unsafe { q::JS_Ext_ValueGetTag(bigint_function) };
    assert_eq!(tag, q::JS_TAG_OBJECT);
    unsafe { q::JS_FreeValue(context, global) };
    bigint_function
}

pub fn create_undefined() -> q::JSValue {
    unsafe { q::JS_Ext_NewSpecialValue(q::JS_TAG_UNDEFINED, 0) }
}

pub fn create_null() -> q::JSValue {
    unsafe { q::JS_Ext_NewSpecialValue(q::JS_TAG_NULL, 0) }
}

pub fn create_bool(context: *mut q::JSContext, value: bool) -> q::JSValue {
    unsafe { q::JS_Ext_NewBool(context, value as u8) }
}

pub fn create_int(context: *mut q::JSContext, value: i32) -> q::JSValue {
    unsafe { q::JS_Ext_NewInt32(context, value) }
}

pub fn create_float(context: *mut q::JSContext, value: f64) -> q::JSValue {
    unsafe { q::JS_Ext_NewFloat64(context, value) }
}

pub fn create_string(context: *mut q::JSContext, value: &str) -> Result<q::JSValue, ValueError> {
    // although rust string is not null-terminated, but quickjs not require it to be null-terminated
    let qval = unsafe { q::JS_NewStringLen(context, value.as_ptr() as *const _, value.len()) };

    let tag = unsafe { q::JS_Ext_ValueGetTag(qval) };

    if tag == q::JS_TAG_EXCEPTION {
        return Err(ValueError::Internal(
            "Could not create string in runtime".into(),
        ));
    }

    Ok(qval)
}

pub fn create_empty_array(context: *mut q::JSContext) -> Result<q::JSValue, ValueError> {
    // Allocate a new array in the runtime.
    let arr = unsafe { q::JS_NewArray(context) };
    let tag = unsafe { q::JS_Ext_ValueGetTag(arr) };
    if tag == q::JS_TAG_EXCEPTION {
        return Err(ValueError::Internal(
            "Could not create array in runtime".into(),
        ));
    }

    Ok(arr)
}

pub fn add_array_element(
    context: *mut q::JSContext,
    array: q::JSValue,
    index: u32,
    value: q::JSValue,
) -> Result<(), ValueError> {
    let result = unsafe { q::JS_SetPropertyUint32(context, array, index, value) };
    if result < 0 {
        return Err(ValueError::Internal(
            "Could not add element to array".into(),
        ));
    }

    Ok(())
}

pub fn create_empty_object(context: *mut q::JSContext) -> Result<q::JSValue, ValueError> {
    let obj = unsafe { q::JS_NewObject(context) };
    let tag = unsafe { q::JS_Ext_ValueGetTag(obj) };
    if tag == q::JS_TAG_EXCEPTION {
        return Err(ValueError::Internal("Could not create object".into()));
    }

    Ok(obj)
}

pub fn add_object_property(
    context: *mut q::JSContext,
    object: q::JSValue,
    key: &str,
    value: q::JSValue,
) -> Result<(), ValueError> {
    let key = make_cstring(key)?;
    let result = unsafe { q::JS_SetPropertyStr(context, object, key.as_ptr(), value) };
    if result < 0 {
        return Err(ValueError::Internal(
            "Could not add property to object".into(),
        ));
    }

    Ok(())
}

pub fn create_function(_: *mut q::JSContext, func: JsFunction) -> Result<q::JSValue, ValueError> {
    let owned_value = func.into_value();
    let v = unsafe { owned_value.extract() };
    Ok(v)
}

#[cfg(feature = "chrono")]
pub fn create_date(
    context: *mut q::JSContext,
    datetime: chrono::DateTime<chrono::Utc>,
) -> Result<q::JSValue, ValueError> {
    let date_constructor = js_date_constructor(context);

    let f = datetime.timestamp_millis() as f64;

    let timestamp = unsafe { q::JS_Ext_NewFloat64(context, f) };

    let mut args = vec![timestamp];

    let value = unsafe {
        q::JS_CallConstructor(
            context,
            date_constructor,
            args.len() as i32,
            args.as_mut_ptr(),
        )
    };
    unsafe {
        q::JS_FreeValue(context, date_constructor);
    }

    let tag = unsafe { q::JS_Ext_ValueGetTag(value) };
    if tag != q::JS_TAG_OBJECT {
        return Err(ValueError::Internal(
            "Could not construct Date object".into(),
        ));
    }
    Ok(value)
}

#[cfg(feature = "bigint")]
pub fn create_bigint(
    context: *mut q::JSContext,
    int: crate::BigInt,
) -> Result<q::JSValue, ValueError> {
    use std::ffi::c_char;

    use crate::value::BigIntOrI64;

    let val = match int.inner {
        BigIntOrI64::Int(int) => unsafe { q::JS_NewBigInt64(context, int) },
        BigIntOrI64::BigInt(bigint) => {
            let bigint_string = bigint.to_str_radix(10);
            let s = unsafe {
                q::JS_NewStringLen(
                    context,
                    bigint_string.as_ptr() as *const c_char,
                    bigint_string.len(),
                )
            };

            let s_tag = unsafe { q::JS_Ext_ValueGetTag(s) };
            if s_tag != q::JS_TAG_STRING {
                return Err(ValueError::Internal(
                    "Could not construct String object needed to create BigInt object".into(),
                ));
            }

            let mut args = vec![s];

            let bigint_function = js_create_bigint_function(context);

            let null = create_null();
            let js_bigint =
                unsafe { q::JS_Call(context, bigint_function, null, 1, args.as_mut_ptr()) };

            unsafe {
                q::JS_FreeValue(context, s);
                q::JS_FreeValue(context, bigint_function);
                q::JS_FreeValue(context, null);
            }

            let js_bigint_tag = unsafe { q::JS_Ext_ValueGetTag(js_bigint) };

            if js_bigint_tag != q::JS_TAG_BIG_INT {
                return Err(ValueError::Internal(
                    "Could not construct BigInt object".into(),
                ));
            }

            js_bigint
        }
    };

    Ok(val)
}

pub fn create_symbol(_: *mut q::JSContext) -> Result<q::JSValue, ValueError> {
    todo!("create symbol not implemented")
}

#[inline]
pub fn own_raw_value(context: *mut q::JSContext, value: q::JSValue) -> OwnedJsValue {
    OwnedJsValue::new(context, value)
}

#[macro_export]
macro_rules! owned {
    ($context:expr, $val:expr) => {
        OwnedJsValue::from(($context, $val))
    };
}

use crate::ExecutionError;

/// Get the last exception from the runtime, and if present, convert it to a ExceptionError.
pub(crate) fn get_exception(context: *mut q::JSContext) -> Option<ExecutionError> {
    if unsafe { !q::JS_HasException(context) } {
        return None;
    }

    let value = unsafe {
        let raw = q::JS_GetException(context);
        OwnedJsValue::new(context, raw)
    };

    if value.is_exception() {
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
