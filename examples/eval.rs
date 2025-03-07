use quickjs_rusty::{Context, JsFunction};

pub fn main() {
    let context = Context::builder()
        .console(|level, args| {
            eprintln!("{}: {:?}", level, args);
        })
        .build()
        .unwrap();

    let value = context.eval("1 + 2", false).unwrap();
    println!("js: 1 + 2 = {:?}", value);

    context.add_callback("myCallback", || 123).unwrap();

    context
        .add_callback("myCallback", |a: i32, b: i32| a + b * b)
        .unwrap();

    context
        .add_callback(
            "myCallbackErr",
            |_: i32, _: i32| -> Result<i32, std::io::Error> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "custom error message",
                ))
            },
        )
        .unwrap();

    context
        .add_callback("test", |func: JsFunction| {
            func.call(vec![]).unwrap();
            func
        })
        .unwrap();

    let value = context
        .eval(
            r#"
       var x = myCallback(10, 20);
       x;
"#,
            false,
        )
        .unwrap();
    println!("js: callback = {:?}", value);

    if let Err(value) = context.eval(
        r#"
        // this will throw an error
        var x = myCallbackErr(10, 20);
        x;
"#,
        false,
    ) {
        // will print `js: callback error = "custom error message"`
        println!("js: callback error = {:?}", value.to_string());
    };

    context
        .eval("const f = test(() => { console.log('hello') }); f()", false)
        .unwrap();
}
