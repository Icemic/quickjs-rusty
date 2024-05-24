use std::ops::Deref;

use libquickjspp_sys as q;

use crate::{ExecutionError, ValueError};

use super::OwnedJsValue;

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
            q::JS_GetLength(
                self.value.context(),
                self.value.value,
                &mut next_index as *mut _,
            );
        }

        next_index as u64
    }

    pub fn get_index(&self, index: u32) -> Result<Option<OwnedJsValue>, ExecutionError> {
        let value_raw =
            unsafe { q::JS_GetPropertyUint32(self.value.context(), self.value.value, index) };
        let tag = unsafe { q::JS_ValueGetTag(value_raw) };
        if tag == q::JS_TAG_EXCEPTION {
            return Err(ExecutionError::Internal("Could not build array".into()));
        } else if tag == q::JS_TAG_UNDEFINED {
            return Ok(None);
        }

        Ok(Some(OwnedJsValue::new(self.value.context(), value_raw)))
    }

    pub fn set_index(&self, index: u32, value: OwnedJsValue) -> Result<(), ExecutionError> {
        unsafe {
            // NOTE: SetPropertyStr takes ownership of the value.
            // We do not, however, call OwnedJsValue::extract immediately, so
            // the inner JSValue is still managed.
            // `mem::forget` is called below only if SetProperty succeeds.
            // This prevents leaks when an error occurs.
            let ret =
                q::JS_SetPropertyUint32(self.value.context(), self.value.value, index, value.value);

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
            q::JS_GetLength(
                self.value.context(),
                self.value.value,
                &mut next_index as *mut _,
            );
            // NOTE: SetPropertyStr takes ownership of the value.
            // We do not, however, call OwnedJsValue::extract immediately, so
            // the inner JSValue is still managed.
            // `mem::forget` is called below only if SetProperty succeeds.
            // This prevents leaks when an error occurs.
            let ret = q::JS_SetPropertyInt64(
                self.value.context(),
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
                unsafe { q::JS_GetPropertyUint32(self.value.context(), self.value.value, i) };
            ret.push(value_raw);
        }
        ret
    }
}

impl Deref for OwnedJsArray {
    type Target = OwnedJsValue;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
