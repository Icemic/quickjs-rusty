use std::ffi::{c_char, c_void, CStr};
use std::ptr::null_mut;

use libquickjspp_sys as q;

use super::compile::compile_module;
use super::ContextWrapper;

pub type JSModuleLoaderFunc = Box<dyn FnMut(&str, *mut c_void) -> String>;
pub type JSModuleNormalizeFunc = Box<dyn FnMut(&str, &str, *mut c_void) -> String>;

pub unsafe extern "C" fn js_module_loader(
    _: *mut q::JSContext,
    module_name: *const c_char,
    opaque: *mut c_void,
) -> *mut q::JSModuleDef {
    let wrapper = &*(opaque as *mut ContextWrapper);
    let opaque = wrapper.module_opaque.lock().unwrap().unwrap_or(null_mut());
    let module_name = CStr::from_ptr(module_name).to_str().unwrap();

    if let Some(module_loader_func) = wrapper.module_loader_func.lock().unwrap().as_mut() {
        let module_script = module_loader_func(module_name, opaque);
        match compile_module(wrapper, &module_script, module_name) {
            Ok(v) => {
                let module_def = q::JS_VALUE_GET_PTR(v.value);
                // q::JS_DupValue(wrapper.context, v.value);
                module_def as *mut q::JSModuleDef
            }
            Err(e) => {
                eprintln!("compile module error: {:?}", e);
                null_mut() as *mut q::JSModuleDef
            }
        }
    } else {
        eprintln!("module loader not set");
        null_mut() as *mut q::JSModuleDef
    }
}

pub unsafe extern "C" fn js_module_normalize(
    ctx: *mut q::JSContext,
    module_base_name: *const c_char,
    module_name: *const c_char,
    opaque: *mut c_void,
) -> *mut c_char {
    let wrapper = &*(opaque as *mut ContextWrapper);
    let opaque = wrapper.module_opaque.lock().unwrap().unwrap_or(null_mut());
    let module_base_name = CStr::from_ptr(module_base_name).to_str().unwrap();
    let module_name = CStr::from_ptr(module_name).to_str().unwrap();

    if let Some(module_normalize_func) = wrapper.module_normalize_func.lock().unwrap().as_mut() {
        let mut normalized_module_name =
            module_normalize_func(module_base_name, module_name, opaque);
        normalized_module_name.push('\0');
        let m = q::js_malloc(ctx, normalized_module_name.len());
        std::ptr::copy(
            normalized_module_name.as_ptr(),
            m as *mut u8,
            normalized_module_name.len(),
        );
        m as *mut c_char
    } else {
        eprintln!("module normalize func not set");
        null_mut() as *mut c_char
    }
}
