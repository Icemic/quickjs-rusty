use std::ffi::c_void;
use std::time::{Duration, SystemTime};

use libquickjs_ng_sys::JSRuntime;
use quickjs_rusty::Context;

const TIMEOUT_MS: u64 = 500;

/// This example shows how to set a timeout interrupt handler to stop long-running JS code.
///
/// This idea was originated from https://github.com/theduke/quickjs-rs/issues/4
pub fn main() {
    let context = Context::builder()
        .console(|level, args| {
            eprintln!("{}: {:?}", level, args);
        })
        .build()
        .unwrap();

    let timeout = Box::new(SystemTime::now() + Duration::from_millis(TIMEOUT_MS));
    let ptr = Box::into_raw(timeout);

    context.set_interrupt_handler(Some(interrupt_handler), ptr as *mut c_void);

    let value = context.eval("1 + 2", false).unwrap();
    println!("eval success: 1 + 2 = {:?}", value);

    let ret = context.eval(
        r#"
      for(;;) {
          console.log("running");
      }
"#,
        false,
    );

    if let Err(e) = ret {
        eprintln!("Error: {:?}", e.to_string());
    }
}

extern "C" fn interrupt_handler(
    _rt: *mut JSRuntime,
    opaque: *mut std::ffi::c_void,
) -> ::std::os::raw::c_int {
    let ts: SystemTime = unsafe { std::ptr::read(opaque as *const SystemTime) };
    if SystemTime::now().gt(&ts) {
        println!("timeout occurred");
        1
    } else {
        0
    }
}
