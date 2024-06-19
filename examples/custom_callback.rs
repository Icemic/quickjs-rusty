use anyhow::Result;
use quickjs_rusty::serde::to_js;
use quickjs_rusty::{owned, Context, JSContext, JsTag, OwnedJsValue, RawJSValue};

pub fn main() {
    let context = Context::builder()
        .console(|_, args| {
            eprintln!("{:?}", args);
        })
        .build()
        .unwrap();
    let ctx = context.context_raw();

    let f = context.create_custom_callback(custom_func).unwrap();

    context.set_global("custom_func", owned!(ctx, f)).unwrap();

    if let Err(err) = context.eval("console.log('returns', custom_func(1, 'haha'))", false) {
        eprintln!("Error: {}", err);
    };
}

fn custom_func(context: *mut JSContext, args: &[RawJSValue]) -> Result<Option<RawJSValue>> {
    let tags: Vec<JsTag> = args
        .iter()
        .map(|arg| OwnedJsValue::own(context, arg).tag())
        .collect();

    println!("custom_func called with parameter type: {:?}", tags);

    let ret = to_js(context, &"This is return".to_string()).unwrap();

    Ok(Some(unsafe { ret.extract() }))
}
