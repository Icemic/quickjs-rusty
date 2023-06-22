use std::ffi::{c_char, c_void, CStr};
use std::ptr::null_mut;

use libquickjspp_sys as q;

use super::make_cstring;

pub type JSModuleLoaderFunc = Box<dyn Fn(&str, *mut c_void) -> String>;
pub type JSModuleNormalizeFunc = Box<dyn Fn(&str, &str, *mut c_void) -> String>;

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

    let module_name = CStr::from_ptr(module_name).to_str().unwrap();
    let module_code = loader(module_name, opaque);

    let module_name_c = match make_cstring(module_name) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("make cstring error: {:?}", e);
            return null_mut() as *mut q::JSModuleDef;
        }
    };
    let module_code_c = match make_cstring(module_code.as_str()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("make cstring error: {:?}", e);
            return null_mut() as *mut q::JSModuleDef;
        }
    };

    let value = unsafe {
        q::JS_Eval(
            ctx,
            module_code_c.as_ptr(),
            module_code.len() as _,
            module_name_c.as_ptr(),
            q::JS_EVAL_TYPE_MODULE as i32 | q::JS_EVAL_FLAG_COMPILE_ONLY as i32,
        )
    };

    // TODO: exception handling

    let module_def = q::JS_VALUE_GET_PTR(value);
    // q::JS_DupValue(wrapper.context, v.value);
    module_def as *mut q::JSModuleDef
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

    println!("2");
    let module_base_name = CStr::from_ptr(module_base_name).to_str().unwrap();
    let module_name = CStr::from_ptr(module_name).to_str().unwrap();

    if let Some(module_normalize_func) = normalize {
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
