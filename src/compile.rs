//! Utils to compile script to bytecode and run script from bytecode

use std::os::raw::c_void;

use libquickjspp_sys as q;

use crate::utils::{ensure_no_excpetion, get_exception, make_cstring};
use crate::value::{JsCompiledFunction, OwnedJsValue};
use crate::ExecutionError;

/// compile a script, will result in a JSValueRef with tag JS_TAG_FUNCTION_BYTECODE or JS_TAG_MODULE.
///  It can be executed with run_compiled_function().
pub fn compile(
    context: *mut q::JSContext,
    script: &str,
    file_name: &str,
) -> Result<OwnedJsValue, ExecutionError> {
    let filename_c = make_cstring(file_name)?;
    let code_c = make_cstring(script)?;

    let value = unsafe {
        let v = q::JS_Eval(
            context,
            code_c.as_ptr(),
            script.len() as _,
            filename_c.as_ptr(),
            q::JS_EVAL_FLAG_COMPILE_ONLY as i32,
        );
        OwnedJsValue::new(context, v)
    };

    // check for error
    ensure_no_excpetion(context)?;
    Ok(value)
}

/// compile a script, will result in a JSValueRef with tag JS_TAG_FUNCTION_BYTECODE or JS_TAG_MODULE.
///  It can be executed with run_compiled_function().
pub fn compile_module(
    context: *mut q::JSContext,
    script: &str,
    file_name: &str,
) -> Result<OwnedJsValue, ExecutionError> {
    let filename_c = make_cstring(file_name)?;
    let code_c = make_cstring(script)?;

    let value = unsafe {
        let v = q::JS_Eval(
            context,
            code_c.as_ptr(),
            script.len() as _,
            filename_c.as_ptr(),
            q::JS_EVAL_TYPE_MODULE as i32 | q::JS_EVAL_FLAG_COMPILE_ONLY as i32,
        );
        OwnedJsValue::new(context, v)
    };

    // check for error
    ensure_no_excpetion(context)?;
    Ok(value)
}

/// run a compiled function, see compile for an example
pub fn run_compiled_function(func: &JsCompiledFunction) -> Result<OwnedJsValue, ExecutionError> {
    let context = func.as_value().context();
    let value = unsafe {
        // NOTE: JS_EvalFunction takes ownership.
        // We clone the func and extract the inner JsValue.
        let f = func.clone().into_value().extract();
        let v = q::JS_EvalFunction(context, f);
        OwnedJsValue::new(context, v)
    };

    ensure_no_excpetion(context).map_err(|e| {
        if let ExecutionError::Internal(msg) = e {
            ExecutionError::Internal(format!("Could not evaluate compiled function: {}", msg))
        } else {
            e
        }
    })?;

    Ok(value)
}

/// write a function to bytecode
pub fn to_bytecode(context: *mut q::JSContext, compiled_func: &JsCompiledFunction) -> Vec<u8> {
    unsafe {
        let mut len = 0;
        let raw = q::JS_WriteObject(
            context,
            &mut len,
            *compiled_func.as_value().as_inner(),
            q::JS_WRITE_OBJ_BYTECODE as i32,
        );
        let slice = std::slice::from_raw_parts(raw, len as usize);
        let data = slice.to_vec();
        q::js_free(context, raw as *mut c_void);
        data
    }
}

/// read a compiled function from bytecode, see to_bytecode for an example
pub fn from_bytecode(
    context: *mut q::JSContext,
    bytecode: &[u8],
) -> Result<OwnedJsValue, ExecutionError> {
    assert!(!bytecode.is_empty());
    {
        let len = bytecode.len();
        let buf = bytecode.as_ptr();
        let raw =
            unsafe { q::JS_ReadObject(context, buf, len as _, q::JS_READ_OBJ_BYTECODE as i32) };

        let func_ref = OwnedJsValue::new(context, raw);
        if func_ref.is_exception() {
            let ex_opt = get_exception(context);
            if let Some(ex) = ex_opt {
                Err(ex)
            } else {
                Err(ExecutionError::Internal(
                    "from_bytecode failed and could not get exception".to_string(),
                ))
            }
        } else {
            Ok(func_ref)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::ContextWrapper;

    #[test]
    fn test_compile_function() {
        let ctx = ContextWrapper::new(None).unwrap();

        let func_res = compile(
            ctx.context,
            "{let a_tb3 = 7; let b_tb3 = 5; a_tb3 * b_tb3;}",
            "test_func.es",
        );
        let func = func_res
            .ok()
            .expect("func compile failed")
            .try_into_compiled_function()
            .unwrap();
        let bytecode: Vec<u8> = to_bytecode(ctx.context, &func);
        drop(func);
        assert!(!bytecode.is_empty());

        let func2_res = from_bytecode(ctx.context, &bytecode);
        let func2 = func2_res
            .ok()
            .expect("could not read bytecode")
            .try_into_compiled_function()
            .unwrap();
        let run_res = run_compiled_function(&func2);
        match run_res {
            Ok(res) => {
                assert_eq!(res.to_int().unwrap(), 7 * 5);
            }
            Err(e) => {
                panic!("run failed1: {}", e);
            }
        }
    }

    #[test]
    fn test_load_and_eval_compiled_function() {
        let ctx = ContextWrapper::new(None).unwrap();

        let func_res = compile(
            ctx.context,
            "{let a_tb4 = 7; let b_tb4 = 5; a_tb4 * b_tb4;}",
            "test_func.es",
        );
        let func = func_res
            .ok()
            .expect("func compile failed")
            .try_into_compiled_function()
            .unwrap();
        let bytecode: Vec<u8> = to_bytecode(ctx.context, &func);
        drop(func);
        assert!(!bytecode.is_empty());
        let func2_res = from_bytecode(ctx.context, &bytecode);
        let func2 = func2_res
            .ok()
            .expect("could not read bytecode")
            .try_into_compiled_function()
            .unwrap();
        let run_res = run_compiled_function(&func2);

        match run_res {
            Ok(res) => {
                assert_eq!(res.to_int().unwrap(), 7 * 5);
            }
            Err(e) => {
                panic!("run failed: {}", e);
            }
        }
    }

    #[test]
    fn test_load_compiled_function_fail() {
        let ctx = ContextWrapper::new(None).unwrap();

        let func_res = compile(
            ctx.context,
            "{the changes of me compil1ng a're slim to 0-0}",
            "test_func_fail.es",
        );
        func_res.err().expect("func compiled unexpectedly");
    }

    #[test]
    fn test_compiled_func_bad_eval() {
        let ctx = ContextWrapper::new(None).unwrap();

        let func_res = compile(ctx.context, "let abcdef = 1;", "test_func_runfail.es");
        let func = func_res
            .ok()
            .expect("func compile failed")
            .try_into_compiled_function()
            .unwrap();
        assert_eq!(1, func.as_value().get_ref_count());

        let bytecode: Vec<u8> = to_bytecode(ctx.context, &func);

        assert_eq!(1, func.as_value().get_ref_count());

        drop(func);

        assert!(!bytecode.is_empty());

        let func2_res = from_bytecode(ctx.context, &bytecode);
        let func2 = func2_res
            .ok()
            .expect("could not read bytecode")
            .try_into_compiled_function()
            .unwrap();

        //should fail the second time you run this because abcdef is already defined

        assert_eq!(1, func2.as_value().get_ref_count());

        let run_res1 = run_compiled_function(&func2)
            .ok()
            .expect("run 1 failed unexpectedly");
        drop(run_res1);

        assert_eq!(1, func2.as_value().get_ref_count());

        let _run_res2 = run_compiled_function(&func2)
            .err()
            .expect("run 2 succeeded unexpectedly");

        assert_eq!(1, func2.as_value().get_ref_count());
    }
}
