use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr::null_mut;

use anyhow::Result;
use libquickjs_ng_sys as q;

use super::compile::compile_module;

/// Custom module loader function, passes (module_name, opaque) and returns module code
/// If the module code is not found, return None
pub type JSModuleLoaderFunc = Box<dyn Fn(&str, *mut c_void) -> Result<String>>;
/// Custom module normalize function, passes (module_base_name, module_name, opaque)
/// and returns normalized module name (or None if not found)
pub type JSModuleNormalizeFunc = Box<dyn Fn(&str, &str, *mut c_void) -> Result<String>>;

pub struct ModuleLoader {
    pub loader: JSModuleLoaderFunc,
    pub normalize: Option<JSModuleNormalizeFunc>,
    pub opaque: *mut c_void,
}

pub unsafe extern "C" fn js_module_loader(
    ctx: *mut q::JSContext,
    module_name: *const c_char,
    opaque: *mut c_void,
) -> *mut q::JSModuleDef {
    let wrapper = &*(opaque as *mut ModuleLoader);
    let opaque = wrapper.opaque;
    let loader = &wrapper.loader;

    let module_name = CStr::from_ptr(module_name).to_string_lossy().to_string();
    let module_code = match loader(&module_name, opaque) {
        Ok(v) => v,
        Err(err) => {
            throw_internal_error(ctx, &err.to_string());
            return null_mut() as *mut q::JSModuleDef;
        }
    };

    match compile_module(ctx, &module_code, &module_name) {
        Ok(v) => {
            let module_def = q::JS_Ext_GetPtr(v.value);
            module_def as *mut q::JSModuleDef
        }
        Err(e) => {
            throw_internal_error(ctx, &e.to_string());
            null_mut() as *mut q::JSModuleDef
        }
    }
}

pub unsafe extern "C" fn js_module_normalize(
    ctx: *mut q::JSContext,
    module_base_name: *const c_char,
    module_name: *const c_char,
    opaque: *mut c_void,
) -> *mut c_char {
    let wrapper = &*(opaque as *mut ModuleLoader);
    let opaque = wrapper.opaque;
    let normalize = &wrapper.normalize;

    let module_base_name = CStr::from_ptr(module_base_name).to_str().unwrap();
    let module_name = CStr::from_ptr(module_name).to_str().unwrap();

    if let Some(module_normalize_func) = normalize {
        let mut normalized_module_name =
            match module_normalize_func(module_base_name, module_name, opaque) {
                Ok(v) => v,
                Err(err) => {
                    throw_internal_error(ctx, &err.to_string());
                    return null_mut() as *mut c_char;
                }
            };
        normalized_module_name.push('\0');
        let m = q::js_malloc(ctx, normalized_module_name.len());
        std::ptr::copy(
            normalized_module_name.as_ptr(),
            m as *mut u8,
            normalized_module_name.len(),
        );
        m as *mut c_char
    } else {
        log::warn!("module normalize func not set");
        null_mut() as *mut c_char
    }
}

#[inline]
fn throw_internal_error(ctx: *mut q::JSContext, err: &str) {
    let err = CString::new(err).unwrap();
    unsafe {
        q::JS_ThrowInternalError(ctx, err.as_ptr() as *const c_char);
    }
}
