use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;
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
use crate::OwnedJsPromise;
use crate::{ExecutionError, ValueError};

use super::tag::JsTag;
use super::JsCompiledFunction;
use super::JsFunction;
use super::JsModule;
use super::OwnedJsArray;
use super::OwnedJsObject;

/// OwnedJsValue wraps a Javascript value owned by the QuickJs runtime.
///
/// Guarantees cleanup of resources by dropping the value from the runtime.
pub struct OwnedJsValue {
    context: *mut q::JSContext,
    // FIXME: make private again, just for testing
    pub(crate) value: q::JSValue,
}

impl PartialEq for OwnedJsValue {
    fn eq(&self, other: &Self) -> bool {
        unsafe { q::JS_VALUE_GET_PTR(self.value) == q::JS_VALUE_GET_PTR(other.value) }
    }
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

    /// Check if this value is a Javascript promise.
    #[inline]
    pub fn is_promise(&self) -> bool {
        unsafe { q::JS_IsPromise(self.context, self.value) == 1 }
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
        use crate::value::BigInt;
        use crate::value::BigIntOrI64;

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

    /// Try convert this value into a function
    pub fn try_into_promise(self) -> Result<OwnedJsPromise, ValueError> {
        OwnedJsPromise::try_from_value(self)
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
        if tag >= q::JS_TAG_FIRST {
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

impl TryFrom<OwnedJsValue> for OwnedJsPromise {
    type Error = ValueError;

    fn try_from(value: OwnedJsValue) -> Result<Self, Self::Error> {
        OwnedJsPromise::try_from_value(value)
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

/// to avoid infinite recursion, we need to implement a ToOwnedJsValue trait for T,
/// and then implement the `From<(*mut q::JSContext, T)>` trait for T and XXX<T> where T: ToOwnedJsValue
///
/// This trait should not be public, use the `From<(*mut q::JSContext, T)>` trait outside of this module.
pub trait ToOwnedJsValue {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue;
}

impl ToOwnedJsValue for bool {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_bool(context, self);
        OwnedJsValue::new(context, val)
    }
}

impl ToOwnedJsValue for i32 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_int(context, self);
        OwnedJsValue::new(context, val)
    }
}

impl ToOwnedJsValue for i8 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_int(context, self as i32);
        OwnedJsValue::new(context, val)
    }
}

impl ToOwnedJsValue for i16 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_int(context, self as i32);
        OwnedJsValue::new(context, val)
    }
}

impl ToOwnedJsValue for u8 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_int(context, self as i32);
        OwnedJsValue::new(context, val)
    }
}

impl ToOwnedJsValue for u16 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_int(context, self as i32);
        OwnedJsValue::new(context, val)
    }
}

impl ToOwnedJsValue for f64 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_float(context, self);
        OwnedJsValue::new(context, val)
    }
}

impl ToOwnedJsValue for u32 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_float(context, self as f64);
        OwnedJsValue::new(context, val)
    }
}

impl ToOwnedJsValue for &str {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_string(context, self).unwrap();
        OwnedJsValue::new(context, val)
    }
}

impl ToOwnedJsValue for String {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_string(context, &self).unwrap();
        OwnedJsValue::new(context, val)
    }
}

#[cfg(feature = "chrono")]
impl ToOwnedJsValue for DateTime<Utc> {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_date(context, self).unwrap();
        OwnedJsValue::new(context, val)
    }
}

#[cfg(feature = "bigint")]
impl ToOwnedJsValue for crate::BigInt {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_bigint(context, self).unwrap();
        OwnedJsValue::new(context, val)
    }
}

#[cfg(feature = "bigint")]
impl ToOwnedJsValue for num_bigint::BigInt {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_bigint(context, self.into()).unwrap();
        OwnedJsValue::new(context, val)
    }
}

#[cfg(feature = "bigint")]
impl ToOwnedJsValue for i64 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_bigint(context, self.into()).unwrap();
        OwnedJsValue::new(context, val)
    }
}

#[cfg(feature = "bigint")]
impl ToOwnedJsValue for u64 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let bigint: num_bigint::BigInt = self.into();
        let val = create_bigint(context, bigint.into()).unwrap();
        OwnedJsValue::new(context, val)
    }
}

#[cfg(feature = "bigint")]
impl ToOwnedJsValue for i128 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let bigint: num_bigint::BigInt = self.into();
        let val = create_bigint(context, bigint.into()).unwrap();
        OwnedJsValue::new(context, val)
    }
}

#[cfg(feature = "bigint")]
impl ToOwnedJsValue for u128 {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let bigint: num_bigint::BigInt = self.into();
        let val = create_bigint(context, bigint.into()).unwrap();
        OwnedJsValue::new(context, val)
    }
}

impl ToOwnedJsValue for JsFunction {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = create_function(context, self).unwrap();
        OwnedJsValue::new(context, val)
    }
}

impl ToOwnedJsValue for OwnedJsPromise {
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let val = unsafe { self.into_value().extract() };
        OwnedJsValue::new(context, val)
    }
}

/// for some cases like HashMap<String, OwnedJsValue>
impl ToOwnedJsValue for OwnedJsValue {
    fn to_owned(self, _: *mut q::JSContext) -> OwnedJsValue {
        self
    }
}

impl<T> ToOwnedJsValue for Vec<T>
where
    T: ToOwnedJsValue,
{
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let arr = create_empty_array(context).unwrap();
        let _ = self.into_iter().enumerate().for_each(|(idx, val)| {
            let val: OwnedJsValue = (context, val).into();
            add_array_element(context, arr, idx as u32, unsafe { val.extract() }).unwrap();
        });

        OwnedJsValue::new(context, arr)
    }
}

impl<K, V> ToOwnedJsValue for HashMap<K, V>
where
    K: Into<String>,
    V: ToOwnedJsValue,
{
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        let obj = create_empty_object(context).unwrap();
        let _ = self.into_iter().for_each(|(key, val)| {
            let val: OwnedJsValue = (context, val).into();
            add_object_property(context, obj, key.into().as_str(), unsafe { val.extract() })
                .unwrap();
        });

        OwnedJsValue::new(context, obj)
    }
}

impl<T> ToOwnedJsValue for Option<T>
where
    T: ToOwnedJsValue,
{
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        if let Some(val) = self {
            (context, val).into()
        } else {
            OwnedJsValue::new(context, create_null())
        }
    }
}

impl<T> ToOwnedJsValue for &T
where
    T: ToOwnedJsValue,
{
    fn to_owned(self, context: *mut q::JSContext) -> OwnedJsValue {
        (context, self).into()
    }
}

impl<T> From<(*mut q::JSContext, T)> for OwnedJsValue
where
    T: ToOwnedJsValue,
{
    fn from((context, value): (*mut q::JSContext, T)) -> Self {
        value.to_owned(context)
    }
}
