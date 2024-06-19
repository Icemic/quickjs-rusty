use std::fmt::Debug;

use libquickjs_ng_sys as q;

#[repr(i32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum JsTag {
    // Used by C code as a marker.
    // Not relevant for bindings.
    // First = q::JS_TAG_FIRST,
    #[cfg(feature = "bigint")]
    BigInt = q::JS_TAG_BIG_INT,
    Symbol = q::JS_TAG_SYMBOL,
    String = q::JS_TAG_STRING,
    Module = q::JS_TAG_MODULE,
    FunctionBytecode = q::JS_TAG_FUNCTION_BYTECODE,
    Object = q::JS_TAG_OBJECT,

    Int = q::JS_TAG_INT,
    Bool = q::JS_TAG_BOOL,
    Null = q::JS_TAG_NULL,
    Undefined = q::JS_TAG_UNDEFINED,
    Uninitialized = q::JS_TAG_UNINITIALIZED,
    CatchOffset = q::JS_TAG_CATCH_OFFSET,
    Exception = q::JS_TAG_EXCEPTION,
    Float64 = q::JS_TAG_FLOAT64,
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
            q::JS_TAG_EXCEPTION => JsTag::Exception,
            q::JS_TAG_UNDEFINED => JsTag::Undefined,
            q::JS_TAG_CATCH_OFFSET => JsTag::CatchOffset,
            q::JS_TAG_UNINITIALIZED => JsTag::Uninitialized,
            q::JS_TAG_FUNCTION_BYTECODE => JsTag::FunctionBytecode,
            #[cfg(feature = "bigint")]
            q::JS_TAG_BIG_INT => JsTag::BigInt,
            _other => {
                unreachable!("Unknown js_tag: {}", _other);
            }
        }
    }

    #[allow(dead_code)]
    pub(super) fn to_c(self) -> i32 {
        // TODO: figure out why this is needed
        // Just casting with `as` does not work correctly
        match self {
            #[cfg(feature = "bigint")]
            JsTag::BigInt => q::JS_TAG_FUNCTION_BYTECODE,
            JsTag::Symbol => q::JS_TAG_SYMBOL,
            JsTag::String => q::JS_TAG_STRING,
            JsTag::Module => q::JS_TAG_MODULE,
            JsTag::FunctionBytecode => q::JS_TAG_FUNCTION_BYTECODE,
            JsTag::Object => q::JS_TAG_OBJECT,

            JsTag::Int => q::JS_TAG_INT,
            JsTag::Bool => q::JS_TAG_BOOL,
            JsTag::Null => q::JS_TAG_NULL,
            JsTag::Undefined => q::JS_TAG_UNDEFINED,
            JsTag::Uninitialized => q::JS_TAG_UNINITIALIZED,
            JsTag::CatchOffset => q::JS_TAG_CATCH_OFFSET,
            JsTag::Exception => q::JS_TAG_EXCEPTION,
            JsTag::Float64 => q::JS_TAG_FLOAT64,
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
}
