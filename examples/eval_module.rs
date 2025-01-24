use anyhow::Result;
use quickjs_rusty::Context;

struct Custom {
    pub foo: i32,
}

pub fn main() {
    let context = Context::builder()
        .console(|level, args| {
            eprintln!("{}: {:?}", level, args);
        })
        .build()
        .unwrap();

    let custom = Custom { foo: 123 };

    context.set_module_loader(
        Box::new(module_loader),
        Some(Box::new(module_normalize)),
        &custom as *const _ as *mut _,
    );

    context.run_module("./m").unwrap();

    let value = context
        .eval_module(
            "import { add } from './m'; console.log('result:', add(1, 2))",
            false,
        )
        .unwrap();
    println!("js: 1 + 2 = {:?}", value);
}

fn module_loader(module_name: &str, opaque: *mut std::ffi::c_void) -> Result<String> {
    println!("module_loader: {:?}", module_name);
    let custom = unsafe { &*(opaque as *mut Custom) };
    assert!(custom.foo == 123);
    Ok("export function add(a, b) { return a + b; }; console.log('module loaded.')".to_string())
}

fn module_normalize(
    module_base_name: &str,
    module_name: &str,
    opaque: *mut std::ffi::c_void,
) -> Result<String> {
    println!("module_normalize: {:?} {:?}", module_base_name, module_name);
    let custom = unsafe { &*(opaque as *mut Custom) };
    assert!(custom.foo == 123);
    Ok(module_name.to_string())
}
