use crate::errors::*;
use crate::value::*;

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
