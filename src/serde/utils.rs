use std::ffi::CString;

use libquickjspp_sys as q;

use crate::bindings::OwnedJsValue;
use crate::{JsFunction, ValueError};

pub(crate) fn deserialize_borrowed_str<'a>(
    context: *mut q::JSContext,
    value: &'a q::JSValue,
) -> Result<&'a str, ValueError> {
    let r = value;
    let tag = unsafe { q::JS_ValueGetTag(*r) };

    match tag {
        q::JS_TAG_STRING => {
            let ptr = unsafe { q::JS_ToCStringLen2(context, std::ptr::null_mut(), *r, 0) };

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
fn make_cstring(value: impl Into<Vec<u8>>) -> Result<CString, ValueError> {
    CString::new(value).map_err(ValueError::StringWithZeroBytes)
}

#[cfg(feature = "chrono")]
fn js_date_constructor(context: *mut q::JSContext) -> q::JSValue {
    let global = unsafe { q::JS_GetGlobalObject(context) };
    let tag = unsafe { q::JS_ValueGetTag(global) };
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
    let tag = unsafe { q::JS_ValueGetTag(date_constructor) };
    assert_eq!(tag, q::JS_TAG_OBJECT);
    unsafe { q::JS_FreeValue(context, global) };
    date_constructor
}

pub(crate) fn create_undefined() -> q::JSValue {
    unsafe { q::JS_NewSpecialValue(q::JS_TAG_UNDEFINED, 0) }
}

pub(crate) fn create_null() -> q::JSValue {
    unsafe { q::JS_NewSpecialValue(q::JS_TAG_NULL, 0) }
}

pub(crate) fn create_bool(context: *mut q::JSContext, value: bool) -> q::JSValue {
    unsafe { q::JS_NewBool(context, value) }
}

pub(crate) fn create_int(context: *mut q::JSContext, value: i32) -> q::JSValue {
    unsafe { q::JS_NewInt32(context, value) }
}

pub(crate) fn create_float(context: *mut q::JSContext, value: f64) -> q::JSValue {
    unsafe { q::JS_NewFloat64(context, value) }
}

pub(crate) fn create_string(
    context: *mut q::JSContext,
    value: &str,
) -> Result<q::JSValue, ValueError> {
    // although rust string is not null-terminated, but quickjs not require it to be null-terminated
    let qval = unsafe { q::JS_NewStringLen(context, value.as_ptr() as *const _, value.len()) };

    let tag = unsafe { q::JS_ValueGetTag(qval) };

    if tag == q::JS_TAG_EXCEPTION {
        return Err(ValueError::Internal(
            "Could not create string in runtime".into(),
        ));
    }

    Ok(qval)
}

pub(crate) fn create_empty_array(context: *mut q::JSContext) -> Result<q::JSValue, ValueError> {
    // Allocate a new array in the runtime.
    let arr = unsafe { q::JS_NewArray(context) };
    let tag = unsafe { q::JS_ValueGetTag(arr) };
    if tag == q::JS_TAG_EXCEPTION {
        return Err(ValueError::Internal(
            "Could not create array in runtime".into(),
        ));
    }

    Ok(arr)
}

pub(crate) fn create_empty_object(context: *mut q::JSContext) -> Result<q::JSValue, ValueError> {
    let obj = unsafe { q::JS_NewObject(context) };
    let tag = unsafe { q::JS_ValueGetTag(obj) };
    if tag == q::JS_TAG_EXCEPTION {
        return Err(ValueError::Internal("Could not create object".into()));
    }

    Ok(obj)
}

pub(crate) fn create_function(
    context: *mut q::JSContext,
    func: JsFunction,
) -> Result<q::JSValue, ValueError> {
    let owned_value = func.into_value();
    let v = unsafe { owned_value.extract() };
    Ok(v)
}

#[cfg(feature = "chrono")]
pub(crate) fn create_date(
    context: *mut q::JSContext,
    datetime: chrono::DateTime<chrono::Utc>,
) -> Result<q::JSValue, ValueError> {
    let date_constructor = js_date_constructor(context);

    let f = datetime.timestamp_millis() as f64;

    let timestamp = unsafe { q::JS_NewFloat64(context, f) };

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

    let tag = unsafe { q::JS_ValueGetTag(value) };
    if tag != q::JS_TAG_OBJECT {
        return Err(ValueError::Internal(
            "Could not construct Date object".into(),
        ));
    }
    Ok(value)
}

#[cfg(feature = "bigint")]
pub(crate) fn create_bigint(
    context: *mut q::JSContext,
    int: num_bigint::BigInt,
) -> Result<q::JSValue, ValueError> {
    match int.to_i64() {
        Some(int) => unsafe { q::JS_NewBigInt64(context, int) },
        None => {
            let bigint_string = int.to_str_radix(10);
            let s = unsafe {
                q::JS_NewStringLen(
                    context,
                    bigint_string.as_ptr() as *const c_char,
                    bigint_string.len() as q::size_t,
                )
            };
            let s = DroppableValue::new(s, |&mut s| unsafe {
                q::JS_FreeValue(context, s);
            });
            if (*s).tag != q::JS_TAG_STRING {
                return Err(ValueError::Internal(
                    "Could not construct String object needed to create BigInt object".into(),
                ));
            }

            let mut args = vec![*s];

            let bigint_function = js_create_bigint_function(context);
            let bigint_function =
                DroppableValue::new(bigint_function, |&mut bigint_function| unsafe {
                    q::JS_FreeValue(context, bigint_function);
                });
            let js_bigint = unsafe {
                q::JS_Call(
                    context,
                    *bigint_function,
                    q::JSValue {
                        u: q::JSValueUnion { int32: 0 },
                        tag: q::JS_TAG_NULL,
                    },
                    1,
                    args.as_mut_ptr(),
                )
            };

            if js_bigint.tag != q::JS_TAG_BIG_INT {
                panic!("Could not construct BigInt object");
            }

            js_bigint
        }
    }
}

pub(crate) fn create_symbol(context: *mut q::JSContext) -> Result<q::JSValue, ValueError> {
    todo!("create symbol not implemented")
}

#[inline]
pub fn own_raw_value(context: *mut q::JSContext, value: q::JSValue) -> OwnedJsValue {
    OwnedJsValue::new(context, value)
}
