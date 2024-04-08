use std::fmt::Debug;

use libquickjspp_sys as q;

use crate::utils::make_cstring;
use crate::{ExecutionError, ValueError};

use super::OwnedJsValue;

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
        let prop_iter = OwnedJsPropertyIterator::from_object(self.value.context(), self.clone())?;

        Ok(prop_iter)
    }

    pub fn property(&self, name: &str) -> Result<Option<OwnedJsValue>, ExecutionError> {
        // TODO: prevent allocation
        let cname = make_cstring(name)?;
        let value = {
            let raw = unsafe {
                q::JS_GetPropertyStr(self.value.context(), self.value.value, cname.as_ptr())
            };
            OwnedJsValue::new(self.value.context(), raw)
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
                self.value.context(),
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
