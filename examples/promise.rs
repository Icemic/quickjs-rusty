use quickjspp::Context;

pub fn main() {
    let context = Context::builder()
        .console(|level, args| {
            eprintln!("{}: {:?}", level, args);
        })
        .build()
        .unwrap();

    let value = context.eval("Promise.resolve()", false).unwrap();

    println!("js is object: {:?}", value.is_object());
    println!("js is promise: {:?}", value.is_promise());
}
