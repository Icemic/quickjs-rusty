use std::ffi::{c_int, c_void};
use std::ptr::null_mut;

use quickjs_rusty::{Context, JSContext, OwnedJsValue, RawJSValue};

/// There's a bug on set_host_promise_rejection_tracker that any rejection no matter if it's handled or not will emit the callback.
///
/// For more information, see https://github.com/quickjs-ng/quickjs/issues/39 and https://github.com/bellard/quickjs/issues/112
///
/// Also, there's a workaround for this issue, see https://github.com/HiRoFa/quickjs_es_runtime/issues/74
pub fn main() {
    let context = Context::builder()
        .console(|level, args| {
            eprintln!("{}: {:?}", level, args);
        })
        .build()
        .unwrap();

    context.set_host_promise_rejection_tracker(Some(host_promise_rejection_tracker), null_mut());

    let value = context.eval("1 + 2", false).unwrap();
    println!("js: 1 + 2 = {:?}", value);

    let ret = context.eval(
        r#"
      async function main() {
                try {
                    // user code here
                    console.log("running");
                    throw Error("test");
                } catch (e) {
                    await 1;
                    throw e;
                }
            }

            // remove the catch block to see the error
            main().catch((e) => {
                console.log("err " + e);
            });
"#,
        false,
    );

    if let Err(e) = ret {
        eprintln!("Error: {:?}", e.to_string());
    }
}

unsafe extern "C" fn host_promise_rejection_tracker(
    ctx: *mut JSContext,
    _promise: RawJSValue,
    reason: RawJSValue,
    is_handled: c_int,
    _opaque: *mut c_void,
) {
    let reason = OwnedJsValue::own(ctx, &reason);
    println!(
        "Promise rejection: {:?}, handled: {}",
        reason.js_to_string(),
        is_handled
    );
}
