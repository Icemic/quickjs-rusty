use std::fmt::Debug;

use libquickjspp_sys as q;

use crate::{ExecutionError, ValueError};

use super::OwnedJsValue;

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
                self.value.context(),
                self.value.value,
                q::JS_NewSpecialValue(q::JS_TAG_NULL, 0),
                qargs.len() as i32,
                qargs.as_mut_ptr(),
            )
        };
        Ok(OwnedJsValue::new(self.value.context(), qres_raw))
    }
}
