//! FFI Bindings for [quickjs-ng](https://github.com/quickjs-ng/quickjs),
//! a Javascript engine.
//! See the [quickjs-rusty](https://crates.io/crates/quickjs-rusty) crate for a high-level
//! wrapper.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use super::*;

    // Small sanity test that starts the runtime and evaluates code.
    #[test]
    fn test_eval() {
        unsafe {
            let rt = JS_NewRuntime();
            let ctx = JS_NewContext(rt);

            let code_str = "1 + 1\0";
            let code = CStr::from_bytes_with_nul(code_str.as_bytes()).unwrap();
            let script = CStr::from_bytes_with_nul("script\0".as_bytes()).unwrap();

            let value = JS_Eval(
                ctx,
                code.as_ptr(),
                code_str.len() - 1,
                script.as_ptr(),
                JS_EVAL_TYPE_GLOBAL as i32,
            );
            assert_eq!(JS_Ext_ValueGetTag(value), JS_TAG_INT);
            assert_eq!(JS_Ext_GetInt(value), 2);

            JS_DupValue(ctx, value);
            JS_FreeValue(ctx, value);

            let ival = JS_Ext_NewInt32(ctx, 12);
            assert_eq!(JS_Ext_ValueGetTag(ival), JS_TAG_INT);
            let fval = JS_Ext_NewFloat64(ctx, f64::MAX);
            assert_eq!(JS_Ext_ValueGetTag(fval), JS_TAG_FLOAT64);
            let bval = JS_Ext_NewBool(ctx, 1);
            assert_eq!(JS_Ext_ValueGetTag(bval), JS_TAG_BOOL);
        }
    }
}
