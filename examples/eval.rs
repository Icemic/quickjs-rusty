use quickjspp::{Context, JsFunction};

pub fn main() {
    let context = Context::builder()
        .console(|level, args| {
            eprintln!("{}: {:?}", level, args);
        })
        .build()
        .unwrap();

    let value = context.eval("1 + 2").unwrap();
    println!("js: 1 + 2 = {:?}", value);

    context.add_callback("myCallback", || 123).unwrap();

    context
        .add_callback("myCallback", |a: i32, b: i32| a + b * b)
        .unwrap();

    context
        .add_callback("test", |func: JsFunction| {
            func.call(vec![]).unwrap();
            return func;
        })
        .unwrap();

    let value = context
        .eval(
            r#"
       var x = myCallback(10, 20);
       x;
"#,
        )
        .unwrap();
    println!("js: callback = {:?}", value);

    context
        .eval("const f = test(() => { console.log('hello') }); f()")
        .unwrap();
}
