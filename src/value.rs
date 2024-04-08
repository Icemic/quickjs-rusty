mod array;
#[cfg(feature = "bigint")]
mod bigint;

mod function;
mod object;
mod tag;
mod value;

use std::fmt::Debug;

use libquickjspp_sys as q;

use crate::{ExecutionError, ValueError};

pub use array::OwnedJsArray;
#[cfg(feature = "bigint")]
pub use bigint::*;
pub use function::*;
pub use object::*;
pub use tag::*;
pub use value::*;

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
        Ok(super::compile::to_bytecode(self.value.context(), self))
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
