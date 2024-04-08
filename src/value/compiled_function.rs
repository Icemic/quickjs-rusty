use crate::errors::*;
use crate::value::*;

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
        crate::compile::run_compiled_function(self)
    }

    /// Convert this compiled function into QuickJS bytecode.
    ///
    /// Bytecode can be stored and loaded with [`Context::compile`].
    // FIXME: add example
    pub fn to_bytecode(&self) -> Result<Vec<u8>, ExecutionError> {
        Ok(crate::compile::to_bytecode(self.value.context(), self))
    }
}
