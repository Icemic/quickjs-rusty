use quickjs_rusty::{Context, OwnedJsPromise};

///
/// This example demonstrates how to create a promise, chain it with callbacks, and handle errors.
///
/// Log:
///
/// ```bash
/// js is object: true
/// js is promise: false
/// fulfilled: 123
/// log: "fulfilled"
/// log: "Error: reject!!!1"
/// ```
pub fn main() {
    let context = Context::builder()
        .console(|level, args: Vec<quickjs_rusty::OwnedJsValue>| {
            println!(
                "{}: {:?}",
                level,
                args.iter()
                    .map(|v| v.js_to_string().unwrap_or("unknown".to_string()))
                    .collect::<Vec<String>>()
                    .join(",")
            );
        })
        .build()
        .unwrap();

    let value = context.eval("() => Promise.resolve(123)", false).unwrap();

    println!("js is object: {:?}", value.is_object());
    println!("js is promise: {:?}", value.is_promise());

    let (promise, resolve, _) = OwnedJsPromise::with_resolvers(&context).unwrap();

    let promise = promise.then(&value).unwrap();

    let on_fulfilled = context
        .create_callback(|aaa: i32| {
            println!("fulfilled: {}", aaa);
            "fulfilled"
        })
        .unwrap();

    let on_fulfilled2 = context
        .eval(
            "(s) => { console.log(s); throw Error('reject!!!1') }",
            false,
        )
        .unwrap();

    let on_rejected = context
        .eval("(err) => { console.log(err.toString()); }", false)
        .unwrap();

    let promise = promise.then(&on_fulfilled).unwrap();
    let promise = promise.then(&on_fulfilled2).unwrap();

    let promise = OwnedJsPromise::all(&context, vec![promise]).unwrap();

    let _ = promise.catch(&on_rejected).unwrap();

    resolve.call(vec![]).unwrap();

    let _ = context.execute_pending_job();
}
