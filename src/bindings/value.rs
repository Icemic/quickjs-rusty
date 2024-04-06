use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt::Debug;
use std::hash::Hash;

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};
use libquickjspp_sys as q;

#[cfg(feature = "bigint")]
use crate::utils::create_bigint;
#[cfg(feature = "chrono")]
use crate::utils::create_date;
use crate::utils::{
    add_array_element, add_object_property, create_bool, create_empty_array, create_empty_object,
    create_float, create_function, create_int, create_null, create_string,
};
use crate::{ExecutionError, ValueError};

use super::make_cstring;

#[repr(u32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum JsTag {
    // Used by C code as a marker.
    // Not relevant for bindings.
    // First = q::JS_TAG_FIRST,
    Int = q::JS_TAG_INT,
    Bool = q::JS_TAG_BOOL,
    Null = q::JS_TAG_NULL,
    Module = q::JS_TAG_MODULE,
    Object = q::JS_TAG_OBJECT,
    String = q::JS_TAG_STRING,
    Symbol = q::JS_TAG_SYMBOL,
    #[cfg(feature = "bigint")]
    BigInt = q::JS_TAG_BIG_INT,
    Float64 = q::JS_TAG_FLOAT64,
    BigFloat = q::JS_TAG_BIG_FLOAT,
    Exception = q::JS_TAG_EXCEPTION,
    Undefined = q::JS_TAG_UNDEFINED,
    BigDecimal = q::JS_TAG_BIG_DECIMAL,
    CatchOffset = q::JS_TAG_CATCH_OFFSET,
    Uninitialized = q::JS_TAG_UNINITIALIZED,
    FunctionBytecode = q::JS_TAG_FUNCTION_BYTECODE,
}

impl JsTag {
    #[inline]
    pub(super) fn from_c(value: &q::JSValue) -> JsTag {
        let inner = unsafe { q::JS_ValueGetTag(*value) };
        match inner {
            q::JS_TAG_INT => JsTag::Int,
            q::JS_TAG_BOOL => JsTag::Bool,
            q::JS_TAG_NULL => JsTag::Null,
            q::JS_TAG_MODULE => JsTag::Module,
            q::JS_TAG_OBJECT => JsTag::Object,
            q::JS_TAG_STRING => JsTag::String,
            q::JS_TAG_SYMBOL => JsTag::Symbol,
            q::JS_TAG_FLOAT64 => JsTag::Float64,
            q::JS_TAG_BIG_FLOAT => JsTag::BigFloat,
            q::JS_TAG_EXCEPTION => JsTag::Exception,
            q::JS_TAG_UNDEFINED => JsTag::Undefined,
            q::JS_TAG_BIG_DECIMAL => JsTag::BigDecimal,
            q::JS_TAG_CATCH_OFFSET => JsTag::CatchOffset,
            q::JS_TAG_UNINITIALIZED => JsTag::Uninitialized,
            q::JS_TAG_FUNCTION_BYTECODE => JsTag::FunctionBytecode,
            #[cfg(feature = "bigint")]
            q::JS_TAG_BIG_INT => JsTag::BigInt,
            _other => {
                unreachable!()
            }
        }
    }

    pub(super) fn to_c(self) -> u32 {
        // TODO: figure out why this is needed
        // Just casting with `as` does not work correctly
        match self {
            JsTag::Int => q::JS_TAG_INT,
            JsTag::Bool => q::JS_TAG_BOOL,
            JsTag::Null => q::JS_TAG_NULL,
            JsTag::Module => q::JS_TAG_MODULE,
            JsTag::Object => q::JS_TAG_OBJECT,
            JsTag::String => q::JS_TAG_STRING,
            JsTag::Symbol => q::JS_TAG_SYMBOL,
            JsTag::Float64 => q::JS_TAG_FLOAT64,
            JsTag::BigFloat => q::JS_TAG_BIG_FLOAT,
            JsTag::Exception => q::JS_TAG_EXCEPTION,
            JsTag::Undefined => q::JS_TAG_UNDEFINED,
            JsTag::BigDecimal => q::JS_TAG_BIG_DECIMAL,
            JsTag::CatchOffset => q::JS_TAG_CATCH_OFFSET,
            JsTag::Uninitialized => q::JS_TAG_UNINITIALIZED,
            JsTag::FunctionBytecode => q::JS_TAG_FUNCTION_BYTECODE,
            #[cfg(feature = "bigint")]
            JsTag::BigInt => q::JS_TAG_FUNCTION_BYTECODE,
        }
    }

    /// Returns `true` if the js_tag is [`Undefined`].
    #[inline]
    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined)
    }

    /// Returns `true` if the js_tag is [`Object`].
    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object)
    }

    /// Returns `true` if the js_tag is [`Exception`].
    #[inline]
    pub fn is_exception(&self) -> bool {
        matches!(self, Self::Exception)
    }

    /// Returns `true` if the js_tag is [`Int`].
    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int)
    }

    /// Returns `true` if the js_tag is [`Bool`].
    #[inline]
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool)
    }

    /// Returns `true` if the js_tag is [`Null`].
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns `true` if the js_tag is [`Module`].
    #[inline]
    pub fn is_module(&self) -> bool {
        matches!(self, Self::Module)
    }

    /// Returns `true` if the js_tag is [`String`].
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String)
    }

    /// Returns `true` if the js_tag is [`Symbol`].
    #[inline]
    pub fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol)
    }

    /// Returns `true` if the js_tag is [`BigInt`].
    #[cfg(feature = "bigint")]
    #[inline]
    pub fn is_big_int(&self) -> bool {
        matches!(self, Self::BigInt)
    }

    /// Returns `true` if the js_tag is [`Float64`].
    #[inline]
    pub fn is_float64(&self) -> bool {
        matches!(self, Self::Float64)
    }

    /// Returns `true` if the js_tag is [`BigFloat`].
    #[inline]
    pub fn is_big_float(&self) -> bool {
        matches!(self, Self::BigFloat)
    }

    /// Returns `true` if the js_tag is [`BigDecimal`].
    #[inline]
    pub fn is_big_decimal(&self) -> bool {
        matches!(self, Self::BigDecimal)
    }
}

pub struct OwnedJsAtom {
    context: *mut q::JSContext,
    value: q::JSAtom,
}

impl OwnedJsAtom {
    #[inline]
    pub fn new(context: *mut q::JSContext, value: q::JSAtom) -> Self {
        Self { context, value }
    }
}

impl Drop for OwnedJsAtom {
    fn drop(&mut self) {
        unsafe {
            q::JS_FreeAtom(self.context, self.value);
        }
    }
}

impl Clone for OwnedJsAtom {
    fn clone(&self) -> Self {
        unsafe { q::JS_DupAtom(self.context, self.value) };
        Self {
            context: self.context,
            value: self.value,
        }
    }
}

/// OwnedJsValue wraps a Javascript value owned by the QuickJs runtime.
///
/// Guarantees cleanup of resources by dropping the value from the runtime.
#[derive(PartialEq)]
pub struct OwnedJsValue {
    context: *mut q::JSContext,
    // FIXME: make private again, just for testing
    pub(crate) value: q::JSValue,
}

impl OwnedJsValue {
    #[inline]
    pub fn context(&self) -> *mut q::JSContext {
        self.context
    }

    /// Create a new `OwnedJsValue` from a `JsValue`.
    /// This will **NOT** increase the ref count of the underlying value. So
    /// you have to manage memory yourself. Be careful when using this.
    #[inline]
    pub fn new(context: *mut q::JSContext, value: q::JSValue) -> Self {
        Self { context, value }
    }

    /// Create a new `OwnedJsValue` from a `JsValue`.
    /// This will increase the ref count of the underlying value.
    #[inline]
    pub fn own(context: *mut q::JSContext, value: &q::JSValue) -> Self {
        unsafe { q::JS_DupValue(context, *value) };
        Self::new(context, *value)
    }

    #[inline]
    pub fn tag(&self) -> JsTag {
        JsTag::from_c(&self.value)
    }

    /// Get the inner JSValue without increasing ref count.
    ///
    /// Unsafe because the caller must ensure proper memory management.
    pub unsafe fn as_inner(&self) -> &q::JSValue {
        &self.value
    }

    /// Extract the underlying JSValue.
    ///
    /// Unsafe because the caller must ensure memory management. (eg JS_FreeValue)
    pub unsafe fn extract(self) -> q::JSValue {
        let v = self.value;
        std::mem::forget(self);
        v
    }

    /// Replace the underlying JSValue.
    /// This will decrease the ref count of the old value but remain the ref count of the new value.
    pub fn replace(&mut self, new: q::JSValue) {
        unsafe {
            q::JS_FreeValue(self.context, self.value);
        }
        self.value = new;
    }

    /// Check if this value is `null`.
    #[inline]
    pub fn is_null(&self) -> bool {
        self.tag().is_null()
    }

    /// Check if this value is `undefined`.
    #[inline]
    pub fn is_undefined(&self) -> bool {
        self.tag() == JsTag::Undefined
    }

    /// Check if this value is `bool`.
    #[inline]
    pub fn is_bool(&self) -> bool {
        self.tag() == JsTag::Bool
    }

    /// Check if this value is `int`.
    #[inline]
    pub fn is_int(&self) -> bool {
        self.tag() == JsTag::Int
    }

    /// Check if this value is `float`.
    #[inline]
    pub fn is_float(&self) -> bool {
        self.tag() == JsTag::Float64
    }

    /// Check if this value is a Javascript exception.
    #[inline]
    pub fn is_exception(&self) -> bool {
        self.tag() == JsTag::Exception
    }

    /// Check if this value is a Javascript object.
    #[inline]
    pub fn is_object(&self) -> bool {
        self.tag() == JsTag::Object
    }

    /// Check if this value is a Javascript array.
    #[inline]
    pub fn is_array(&self) -> bool {
        unsafe { q::JS_IsArray(self.context, self.value) == 1 }
    }

    /// Check if this value is a Javascript function.
    #[inline]
    pub fn is_function(&self) -> bool {
        unsafe { q::JS_IsFunction(self.context, self.value) == 1 }
    }

    /// Check if this value is a Javascript module.
    #[inline]
    pub fn is_module(&self) -> bool {
        self.tag().is_module()
    }

    /// Check if this value is a Javascript string.
    #[inline]
    pub fn is_string(&self) -> bool {
        self.tag() == JsTag::String
    }

    /// Check if this value is a bytecode compiled function.
    #[inline]
    pub fn is_compiled_function(&self) -> bool {
        self.tag() == JsTag::FunctionBytecode
    }

    #[inline]
    fn check_tag(&self, expected: JsTag) -> Result<(), ValueError> {
        if self.tag() == expected {
            Ok(())
        } else {
            Err(ValueError::UnexpectedType)
        }
    }

    /// Convert this value into a bool
    pub fn to_bool(&self) -> Result<bool, ValueError> {
        self.check_tag(JsTag::Bool)?;
        let val = unsafe { q::JS_VALUE_GET_BOOL(self.value) };
        Ok(val)
    }

    /// Convert this value into an i32
    pub fn to_int(&self) -> Result<i32, ValueError> {
        self.check_tag(JsTag::Int)?;
        let val = unsafe { q::JS_VALUE_GET_INT(self.value) };
        Ok(val)
    }

    /// Convert this value into an f64
    pub fn to_float(&self) -> Result<f64, ValueError> {
        self.check_tag(JsTag::Float64)?;
        let val = unsafe { q::JS_VALUE_GET_FLOAT64(self.value) };
        Ok(val)
    }

    /// Convert this value into a string
    pub fn to_string(&self) -> Result<String, ValueError> {
        self.check_tag(JsTag::String)?;
        let ptr = unsafe { q::JS_ToCStringLen2(self.context, std::ptr::null_mut(), self.value, 0) };

        if ptr.is_null() {
            return Err(ValueError::Internal(
                "Could not convert string: got a null pointer".into(),
            ));
        }

        let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };

        let s = cstr
            .to_str()
            .map_err(ValueError::InvalidString)?
            .to_string();

        // Free the c string.
        unsafe { q::JS_FreeCString(self.context, ptr) };

        Ok(s)
    }

    pub fn to_array(&self) -> Result<OwnedJsArray, ValueError> {
        OwnedJsArray::try_from_value(self.clone())
    }

    /// Try convert this value into a object
    pub fn try_into_object(self) -> Result<OwnedJsObject, ValueError> {
        OwnedJsObject::try_from_value(self)
    }

    #[cfg(feature = "chrono")]
    pub fn to_date(&self) -> Result<chrono::DateTime<chrono::Utc>, ValueError> {
        use chrono::offset::TimeZone;

        use crate::utils::js_date_constructor;

        let date_constructor = js_date_constructor(self.context);
        let is_date = unsafe { q::JS_IsInstanceOf(self.context, self.value, date_constructor) > 0 };

        if is_date {
            let getter = unsafe {
                q::JS_GetPropertyStr(
                    self.context,
                    self.value,
                    std::ffi::CStr::from_bytes_with_nul(b"getTime\0")
                        .unwrap()
                        .as_ptr(),
                )
            };
            let tag = unsafe { q::JS_ValueGetTag(getter) };
            assert_eq!(tag, q::JS_TAG_OBJECT);

            let timestamp_raw =
                unsafe { q::JS_Call(self.context, getter, self.value, 0, std::ptr::null_mut()) };

            unsafe {
                q::JS_FreeValue(self.context, getter);
                q::JS_FreeValue(self.context, date_constructor);
            };

            let tag = unsafe { q::JS_ValueGetTag(timestamp_raw) };
            let res = if tag == q::JS_TAG_FLOAT64 {
                let f = unsafe { q::JS_VALUE_GET_FLOAT64(timestamp_raw) } as i64;
                let datetime = chrono::Utc.timestamp_millis_opt(f).unwrap();
                Ok(datetime)
            } else if tag == q::JS_TAG_INT {
                let f = unsafe { q::JS_VALUE_GET_INT(timestamp_raw) } as i64;
                let datetime = chrono::Utc.timestamp_millis_opt(f).unwrap();
                Ok(datetime)
            } else {
                Err(ValueError::Internal(
                    "Could not convert 'Date' instance to timestamp".into(),
                ))
            };
            return res;
        } else {
            unsafe { q::JS_FreeValue(self.context, date_constructor) };
            Err(ValueError::UnexpectedType)
        }
    }

    #[cfg(feature = "bigint")]
    pub fn to_bigint(&self) -> Result<crate::BigInt, ValueError> {
        use crate::bigint::BigIntOrI64;
        use crate::BigInt;

        let mut int: i64 = 0;
        let ret = unsafe { q::JS_ToBigInt64(self.context, &mut int, self.value) };
        if ret == 0 {
            Ok(BigInt {
                inner: BigIntOrI64::Int(int),
            })
        } else {
            let ptr =
                unsafe { q::JS_ToCStringLen2(self.context, std::ptr::null_mut(), self.value, 0) };

            if ptr.is_null() {
                return Err(ValueError::Internal(
                    "Could not convert BigInt to string: got a null pointer".into(),
                ));
            }

            let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
            let bigint = num_bigint::BigInt::parse_bytes(cstr.to_bytes(), 10).unwrap();

            // Free the c string.
            unsafe { q::JS_FreeCString(self.context, ptr) };

            Ok(BigInt {
                inner: BigIntOrI64::BigInt(bigint),
            })
        }
    }

    /// Try convert this value into a function
    pub fn try_into_function(self) -> Result<JsFunction, ValueError> {
        JsFunction::try_from_value(self)
    }

    /// Try convert this value into a compiled function
    pub fn try_into_compiled_function(self) -> Result<JsCompiledFunction, ValueError> {
        JsCompiledFunction::try_from_value(self)
    }

    /// Try convert this value into a module
    pub fn try_into_module(self) -> Result<JsModule, ValueError> {
        JsModule::try_from_value(self)
    }

    /// Call the Javascript `.toString()` method on this value.
    pub fn js_to_string(&self) -> Result<String, ExecutionError> {
        let value = if self.is_string() {
            self.to_string()?
        } else {
            let raw = unsafe { q::JS_ToString(self.context, self.value) };
            let value = OwnedJsValue::new(self.context, raw);

            if !value.is_string() {
                return Err(ExecutionError::Internal(
                    "Could not convert value to string".into(),
                ));
            }
            value.to_string()?
        };

        Ok(value)
    }

    /// Call the Javascript `JSON.stringify()` method on this value.
    pub fn to_json_string(&self, space: u8) -> Result<String, ExecutionError> {
        let replacer = unsafe { q::JS_NewSpecialValue(q::JS_TAG_NULL, 0) };
        let space = unsafe { q::JS_NewInt32(self.context, space as i32) };
        let raw = unsafe { q::JS_JSONStringify(self.context, self.value, replacer, space) };

        let value = OwnedJsValue::new(self.context, raw);

        unsafe {
            q::JS_FreeValue(self.context, replacer);
            q::JS_FreeValue(self.context, space);
        }

        if !value.is_string() {
            return Err(ExecutionError::Internal(
                "Could not convert value to string".to_string(),
            ));
        }

        let value = value.to_string()?;

        Ok(value)
    }

    #[cfg(test)]
    pub(crate) fn get_ref_count(&self) -> i32 {
        let tag = unsafe { q::JS_ValueGetTag(self.value) };
        if tag >= 8 {
            // This transmute is OK since if tag < 0, the union will be a refcount
            // pointer.
            let ptr = unsafe { q::JS_VALUE_GET_PTR(self.value) as *mut q::JSRefCountHeader };
            let pref: &mut q::JSRefCountHeader = &mut unsafe { *ptr };
            pref.ref_count
        } else {
            -1
        }
    }
}

impl Drop for OwnedJsValue {
    fn drop(&mut self) {
        unsafe {
            q::JS_FreeValue(self.context, self.value);
        }
    }
}

impl Clone for OwnedJsValue {
    fn clone(&self) -> Self {
        unsafe { q::JS_DupValue(self.context, self.value) };
        Self {
            context: self.context,
            value: self.value,
        }
    }
}

impl std::fmt::Debug for OwnedJsValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}(_)", self.tag())
    }
}

impl TryFrom<OwnedJsValue> for bool {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        value.to_bool()
    }
}

impl TryFrom<OwnedJsValue> for i32 {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        value.to_int()
    }
}

impl TryFrom<OwnedJsValue> for f64 {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        value.to_float()
    }
}

impl TryFrom<OwnedJsValue> for String {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        value.to_string()
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<OwnedJsValue> for DateTime<Utc> {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        value.to_date()
    }
}

#[cfg(feature = "bigint")]
impl TryFrom<OwnedJsValue> for crate::BigInt {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        value.to_bigint()
    }
}

#[cfg(feature = "bigint")]
impl TryFrom<OwnedJsValue> for i64 {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        value.to_bigint().map(|v| v.as_i64().unwrap())
    }
}

#[cfg(feature = "bigint")]
impl TryFrom<OwnedJsValue> for u64 {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        use num_traits::ToPrimitive;
        let bigint = value.to_bigint()?;
        bigint
            .into_bigint()
            .to_u64()
            .ok_or(ValueError::BigIntOverflow)
    }
}

#[cfg(feature = "bigint")]
impl TryFrom<OwnedJsValue> for i128 {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        use num_traits::ToPrimitive;
        let bigint = value.to_bigint()?;
        bigint
            .into_bigint()
            .to_i128()
            .ok_or(ValueError::BigIntOverflow)
    }
}

#[cfg(feature = "bigint")]
impl TryFrom<OwnedJsValue> for u128 {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        use num_traits::ToPrimitive;
        let bigint = value.to_bigint()?;
        bigint
            .into_bigint()
            .to_u128()
            .ok_or(ValueError::BigIntOverflow)
    }
}

#[cfg(feature = "bigint")]
impl TryFrom<OwnedJsValue> for num_bigint::BigInt {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        value.to_bigint().map(|v| v.into_bigint())
    }
}

impl<T: TryFrom<OwnedJsValue, Error = ValueError>> TryFrom<OwnedJsValue> for Vec<T> {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        let arr = value.to_array()?;
        let mut ret: Vec<T> = vec![];
        for i in 0..arr.length() {
            let item = arr.get_index(i as u32).unwrap();
            if let Some(item) = item {
                let item = item.try_into()?;
                ret.push(item);
            }
        }
        Ok(ret)
    }
}

impl<K: From<String> + PartialEq + Eq + Hash, V: TryFrom<OwnedJsValue, Error = ValueError>>
    TryFrom<OwnedJsValue> for HashMap<K, V>
{
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        let obj = value.try_into_object()?;
        let mut ret: HashMap<K, V> = HashMap::new();
        let mut iter = obj.properties_iter()?;
        while let Some(Ok(key)) = iter.next() {
            let key = key.to_string()?;
            let item = obj.property(&key).unwrap();
            if let Some(item) = item {
                let item = item.try_into()?;
                ret.insert(key.into(), item);
            }
        }
        Ok(ret)
    }
}

impl TryFrom<OwnedJsValue> for JsFunction {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        JsFunction::try_from_value(value)
    }
}

impl TryFrom<OwnedJsValue> for OwnedJsArray {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        OwnedJsArray::try_from_value(value)
    }
}

impl TryFrom<OwnedJsValue> for OwnedJsObject {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        OwnedJsObject::try_from_value(value)
    }
}

impl TryFrom<OwnedJsValue> for JsCompiledFunction {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        JsCompiledFunction::try_from_value(value)
    }
}

impl TryFrom<OwnedJsValue> for JsModule {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        JsModule::try_from_value(value)
    }
}

pub struct OwnedJsArray {
    value: OwnedJsValue,
}

impl OwnedJsArray {
    pub fn try_from_value(value: OwnedJsValue) -> Result<Self, ValueError> {
        if !value.is_array() {
            Err(ValueError::Internal("Expected an array".into()))
        } else {
            Ok(Self { value })
        }
    }

    pub fn length(&self) -> u64 {
        let mut next_index: i64 = 0;
        unsafe {
            q::JS_GetPropertyLength(
                self.value.context,
                &mut next_index as *mut _,
                self.value.value,
            );
        }

        next_index as u64
    }

    pub fn get_index(&self, index: u32) -> Result<Option<OwnedJsValue>, ExecutionError> {
        let value_raw =
            unsafe { q::JS_GetPropertyUint32(self.value.context, self.value.value, index) };
        let tag = unsafe { q::JS_ValueGetTag(value_raw) };
        if tag == q::JS_TAG_EXCEPTION {
            return Err(ExecutionError::Internal("Could not build array".into()));
        } else if tag == q::JS_TAG_UNDEFINED {
            return Ok(None);
        }

        Ok(Some(OwnedJsValue::new(self.value.context, value_raw)))
    }

    pub fn set_index(&self, index: u32, value: OwnedJsValue) -> Result<(), ExecutionError> {
        unsafe {
            // NOTE: SetPropertyStr takes ownership of the value.
            // We do not, however, call OwnedJsValue::extract immediately, so
            // the inner JSValue is still managed.
            // `mem::forget` is called below only if SetProperty succeeds.
            // This prevents leaks when an error occurs.
            let ret =
                q::JS_SetPropertyUint32(self.value.context, self.value.value, index, value.value);

            if ret < 0 {
                Err(ExecutionError::Internal("Could not set property".into()))
            } else {
                // Now we can call forget to prevent calling the destructor.
                std::mem::forget(value);
                Ok(())
            }
        }
    }

    pub fn push(&self, value: OwnedJsValue) -> Result<(), ExecutionError> {
        unsafe {
            let mut next_index: i64 = 0;
            q::JS_GetPropertyLength(
                self.value.context,
                &mut next_index as *mut _,
                self.value.value,
            );
            // NOTE: SetPropertyStr takes ownership of the value.
            // We do not, however, call OwnedJsValue::extract immediately, so
            // the inner JSValue is still managed.
            // `mem::forget` is called below only if SetProperty succeeds.
            // This prevents leaks when an error occurs.
            let ret = q::JS_SetPropertyInt64(
                self.value.context,
                self.value.value,
                next_index,
                value.value,
            );

            if ret < 0 {
                Err(ExecutionError::Internal(
                    "Could not set property".to_string(),
                ))
            } else {
                // Now we can call forget to prevent calling the destructor.
                std::mem::forget(value);
                Ok(())
            }
        }
    }

    pub fn raw_elements(&self) -> Vec<q::JSValue> {
        let mut ret = vec![];
        let length = self.length() as u32;
        for i in 0..length {
            let value_raw =
                unsafe { q::JS_GetPropertyUint32(self.value.context, self.value.value, i) };
            ret.push(value_raw);
        }
        ret
    }
}

/// Wraps an object from the QuickJs runtime.
/// Provides convenience property accessors.
#[derive(Clone, Debug, PartialEq)]
pub struct OwnedJsObject {
    value: OwnedJsValue,
}

impl OwnedJsObject {
    pub fn try_from_value(value: OwnedJsValue) -> Result<Self, ValueError> {
        if !value.is_object() {
            Err(ValueError::Internal("Expected an object".to_string()))
        } else {
            Ok(Self { value })
        }
    }

    pub fn into_value(self) -> OwnedJsValue {
        self.value
    }

    pub fn properties_iter(&self) -> Result<OwnedJsPropertyIterator, ValueError> {
        let prop_iter = OwnedJsPropertyIterator::from_object(self.value.context, self.clone())?;

        Ok(prop_iter)
    }

    pub fn property(&self, name: &str) -> Result<Option<OwnedJsValue>, ExecutionError> {
        // TODO: prevent allocation
        let cname = make_cstring(name)?;
        let value = {
            let raw = unsafe {
                q::JS_GetPropertyStr(self.value.context, self.value.value, cname.as_ptr())
            };
            OwnedJsValue::new(self.value.context, raw)
        };
        let tag = value.tag();

        if tag.is_exception() {
            Err(ExecutionError::Internal(format!(
                "Exception while getting property '{}'",
                name
            )))
        }
        //  else if tag.is_undefined() {
        //     Ok(None)
        // }
        else {
            Ok(Some(value))
        }
    }

    pub fn property_require(&self, name: &str) -> Result<OwnedJsValue, ExecutionError> {
        self.property(name)?
            .ok_or_else(|| ExecutionError::Internal(format!("Property '{}' not found", name)))
    }

    /// Determine if the object is a promise by checking the presence of
    /// a 'then' and a 'catch' property.
    pub fn is_promise(&self) -> Result<bool, ExecutionError> {
        if let Some(p) = self.property("then")? {
            if p.is_function() {
                return Ok(true);
            }
        }
        if let Some(p) = self.property("catch")? {
            if p.is_function() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn set_property(&self, name: &str, value: OwnedJsValue) -> Result<(), ExecutionError> {
        let cname = make_cstring(name)?;
        unsafe {
            // NOTE: SetPropertyStr takes ownership of the value.
            // We do not, however, call OwnedJsValue::extract immediately, so
            // the inner JSValue is still managed.
            // `mem::forget` is called below only if SetProperty succeeds.
            // This prevents leaks when an error occurs.
            let ret = q::JS_SetPropertyStr(
                self.value.context,
                self.value.value,
                cname.as_ptr(),
                value.value,
            );

            if ret < 0 {
                Err(ExecutionError::Internal(
                    "Could not set property".to_string(),
                ))
            } else {
                // Now we can call forget to prevent calling the destructor.
                std::mem::forget(value);
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OwnedJsPropertyIterator {
    context: *mut q::JSContext,
    object: OwnedJsObject,
    properties: *mut q::JSPropertyEnum,
    length: u32,
    cur_index: u32,
}

impl OwnedJsPropertyIterator {
    pub fn from_object(
        context: *mut q::JSContext,
        object: OwnedJsObject,
    ) -> Result<Self, ValueError> {
        let mut properties: *mut q::JSPropertyEnum = std::ptr::null_mut();
        let mut length: u32 = 0;

        let flags = (q::JS_GPN_STRING_MASK | q::JS_GPN_SYMBOL_MASK | q::JS_GPN_ENUM_ONLY) as i32;
        let ret = unsafe {
            q::JS_GetOwnPropertyNames(
                context,
                &mut properties,
                &mut length,
                object.value.value,
                flags,
            )
        };
        if ret != 0 {
            return Err(ValueError::Internal(
                "Could not get object properties".into(),
            ));
        }

        Ok(Self {
            context,
            object,
            properties,
            length,
            cur_index: 0,
        })
    }
}

/// Iterator over the properties of an object.
/// The iterator yields key first and then value.
impl Iterator for OwnedJsPropertyIterator {
    type Item = Result<OwnedJsValue, ExecutionError>;

    fn next(&mut self) -> Option<Self::Item> {
        let cur_index = self.cur_index / 2;
        let is_key = (self.cur_index % 2) == 0;

        if cur_index >= self.length {
            return None;
        }

        let prop = unsafe { self.properties.offset(cur_index as isize) };

        let value = if is_key {
            let pair_key = unsafe { q::JS_AtomToString(self.context, (*prop).atom) };
            let tag = unsafe { q::JS_ValueGetTag(pair_key) };
            if tag == q::JS_TAG_EXCEPTION {
                return Some(Err(ExecutionError::Internal(
                    "Could not get object property name".into(),
                )));
            }

            OwnedJsValue::new(self.context, pair_key)
        } else {
            let pair_value = unsafe {
                q::JS_GetPropertyInternal(
                    self.context,
                    self.object.value.value,
                    (*prop).atom,
                    self.object.value.value,
                    0,
                )
            };
            let tag = unsafe { q::JS_ValueGetTag(pair_value) };
            if tag == q::JS_TAG_EXCEPTION {
                return Some(Err(ExecutionError::Internal(
                    "Could not get object property".into(),
                )));
            }

            OwnedJsValue::new(self.context, pair_value)
        };

        self.cur_index += 1;

        Some(Ok(value))
    }
}

impl Drop for OwnedJsPropertyIterator {
    fn drop(&mut self) {
        unsafe {
            q::js_free_prop_enum(self.context, self.properties, self.length);
        }
    }
}

/// Wraps an object from the QuickJs runtime.
/// Provides convenience property accessors.
#[derive(Clone, Debug, PartialEq)]
pub struct JsFunction {
    value: OwnedJsValue,
}

impl JsFunction {
    pub fn try_from_value(value: OwnedJsValue) -> Result<Self, ValueError> {
        if !value.is_function() {
            Err(ValueError::Internal(format!(
                "Expected a function, got {:?}",
                value.tag()
            )))
        } else {
            Ok(Self { value })
        }
    }

    pub fn into_value(self) -> OwnedJsValue {
        self.value
    }

    pub fn call(&self, args: Vec<OwnedJsValue>) -> Result<OwnedJsValue, ExecutionError> {
        let mut qargs = args.iter().map(|arg| arg.value).collect::<Vec<_>>();

        let qres_raw = unsafe {
            q::JS_Call(
                self.value.context,
                self.value.value,
                q::JS_NewSpecialValue(q::JS_TAG_NULL, 0),
                qargs.len() as i32,
                qargs.as_mut_ptr(),
            )
        };
        Ok(OwnedJsValue::new(self.value.context, qres_raw))
    }
}

/// A bytecode compiled function.
#[derive(Clone, Debug)]
pub struct JsCompiledFunction {
    value: OwnedJsValue,
}

impl JsCompiledFunction {
    pub(crate) fn try_from_value(value: OwnedJsValue) -> Result<Self, ValueError> {
        if !value.is_compiled_function() {
            Err(ValueError::Internal(format!(
                "Expected a compiled function, got {:?}",
                value.tag()
            )))
        } else {
            Ok(Self { value })
        }
    }

    pub(crate) fn as_value(&self) -> &OwnedJsValue {
        &self.value
    }

    pub(crate) fn into_value(self) -> OwnedJsValue {
        self.value
    }

    /// Evaluate this compiled function and return the resulting value.
    // FIXME: add example
    pub fn eval(&self) -> Result<OwnedJsValue, ExecutionError> {
        super::compile::run_compiled_function(self)
    }

    /// Convert this compiled function into QuickJS bytecode.
    ///
    /// Bytecode can be stored and loaded with [`Context::compile`].
    // FIXME: add example
    pub fn to_bytecode(&self) -> Result<Vec<u8>, ExecutionError> {
        Ok(super::compile::to_bytecode(self.value.context, self))
    }
}

/// A bytecode compiled module.
pub struct JsModule {
    value: OwnedJsValue,
}

impl JsModule {
    pub fn try_from_value(value: OwnedJsValue) -> Result<Self, ValueError> {
        if !value.is_module() {
            Err(ValueError::Internal(format!(
                "Expected a compiled function, got {:?}",
                value.tag()
            )))
        } else {
            Ok(Self { value })
        }
    }

    pub fn into_value(self) -> OwnedJsValue {
        self.value
    }
}

/// The result of loading QuickJs bytecode.
/// Either a function or a module.
pub enum JsCompiledValue {
    Function(JsCompiledFunction),
    Module(JsModule),
}

/// to avoid infinite recursion, we need to implement a PrimitiveToOwnedJsValue trait for T,
/// and then implement the `From<(*mut q::JSContext, T)>` trait for T and XXX<T> where T: PrimitiveToOwnedJsValue
///
/// This trait should not be public, use the `From<(*mut q::JSContext, T)>` trait outside of this module.
trait PrimitiveToOwnedJsValue {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue;
}

impl PrimitiveToOwnedJsValue for bool {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_bool(context, self);
        OwnedJsValue::new(context, val)
    }
}

impl PrimitiveToOwnedJsValue for i32 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_int(context, self);
        OwnedJsValue::new(context, val)
    }
}

impl PrimitiveToOwnedJsValue for i8 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_int(context, self as i32);
        OwnedJsValue::new(context, val)
    }
}

impl PrimitiveToOwnedJsValue for i16 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_int(context, self as i32);
        OwnedJsValue::new(context, val)
    }
}

impl PrimitiveToOwnedJsValue for u8 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_int(context, self as i32);
        OwnedJsValue::new(context, val)
    }
}

impl PrimitiveToOwnedJsValue for u16 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_int(context, self as i32);
        OwnedJsValue::new(context, val)
    }
}

impl PrimitiveToOwnedJsValue for f64 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_float(context, self);
        OwnedJsValue::new(context, val)
    }
}

impl PrimitiveToOwnedJsValue for u32 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_float(context, self as f64);
        OwnedJsValue::new(context, val)
    }
}

impl PrimitiveToOwnedJsValue for &str {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_string(context, self).unwrap();
        OwnedJsValue::new(context, val)
    }
}

impl PrimitiveToOwnedJsValue for String {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_string(context, &self).unwrap();
        OwnedJsValue::new(context, val)
    }
}

#[cfg(feature = "chrono")]
impl PrimitiveToOwnedJsValue for DateTime<Utc> {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_date(context, self).unwrap();
        OwnedJsValue::new(context, val)
    }
}

#[cfg(feature = "bigint")]
impl PrimitiveToOwnedJsValue for crate::BigInt {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_bigint(context, self).unwrap();
        OwnedJsValue::new(context, val)
    }
}

#[cfg(feature = "bigint")]
impl PrimitiveToOwnedJsValue for num_bigint::BigInt {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_bigint(context, self.into()).unwrap();
        OwnedJsValue::new(context, val)
    }
}

#[cfg(feature = "bigint")]
impl PrimitiveToOwnedJsValue for i64 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_bigint(context, self.into()).unwrap();
        OwnedJsValue::new(context, val)
    }
}

#[cfg(feature = "bigint")]
impl PrimitiveToOwnedJsValue for u64 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let bigint: num_bigint::BigInt = self.into();
        let val = create_bigint(context, bigint.into()).unwrap();
        OwnedJsValue::new(context, val)
    }
}

#[cfg(feature = "bigint")]
impl PrimitiveToOwnedJsValue for i128 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let bigint: num_bigint::BigInt = self.into();
        let val = create_bigint(context, bigint.into()).unwrap();
        OwnedJsValue::new(context, val)
    }
}

#[cfg(feature = "bigint")]
impl PrimitiveToOwnedJsValue for u128 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let bigint: num_bigint::BigInt = self.into();
        let val = create_bigint(context, bigint.into()).unwrap();
        OwnedJsValue::new(context, val)
    }
}

impl PrimitiveToOwnedJsValue for JsFunction {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_function(context, self).unwrap();
        OwnedJsValue::new(context, val)
    }
}

/// for some cases like HashMap<String, OwnedJsValue>
impl PrimitiveToOwnedJsValue for OwnedJsValue {
    fn to_owned(self, _: *mut q::JSContext) -> OwnedJsValue {
        self
    }
}

impl<T> From<(*mut q::JSContext, T)> for OwnedJsValue
where
    T: PrimitiveToOwnedJsValue,
{
    fn from((context, value): (*mut q::JSContext, T)) -> Self {
        value.to_owned(context)
    }
}

impl<T> From<(*mut q::JSContext, Option<T>)> for OwnedJsValue
where
    T: PrimitiveToOwnedJsValue,
{
    fn from((context, value): (*mut q::JSContext, Option<T>)) -> Self {
        if let Some(val) = value {
            (context, val).into()
        } else {
            OwnedJsValue::new(context, create_null())
        }
    }
}

impl<T: Debug> From<(*mut q::JSContext, Vec<T>)> for OwnedJsValue
where
    T: PrimitiveToOwnedJsValue,
{
    fn from((context, values): (*mut q::JSContext, Vec<T>)) -> Self {
        let arr = create_empty_array(context).unwrap();
        let _ = values.into_iter().enumerate().for_each(|(idx, val)| {
            let val: OwnedJsValue = (context, val).into();
            add_array_element(context, arr, idx as u32, unsafe { val.extract() }).unwrap();
        });

        OwnedJsValue::new(context, arr)
    }
}

impl<K, V> From<(*mut q::JSContext, HashMap<K, V>)> for OwnedJsValue
where
    K: Into<String>,
    V: PrimitiveToOwnedJsValue,
{
    fn from((context, values): (*mut q::JSContext, HashMap<K, V>)) -> Self {
        let obj = create_empty_object(context).unwrap();
        let _ = values.into_iter().for_each(|(key, val)| {
            let val: OwnedJsValue = (context, val).into();
            add_object_property(context, obj, key.into().as_str(), unsafe { val.extract() })
                .unwrap();
        });

        OwnedJsValue::new(context, obj)
    }
}
