use std::collections::HashMap;

use quickjs_rusty::value::*;
use quickjs_rusty::*;

// #[test]
// fn test_global_properties() {
//     let c = Context::builder().build().unwrap();
// let ctx = c.context_raw();

//     assert_eq!(
//         c.global_property("lala"),
//         Err(ExecutionError::Internal(
//             "Global object does not have property 'lala'".to_string()
//         ))
//     );

//     c.set_global_property("testprop", true).unwrap();
//     assert_eq!(
//         c.global_property("testprop").unwrap(),
//         JsValue::Bool(true),
//     );
// }

#[test]
fn test_eval_pass() {
    let c = Context::builder().build().unwrap();

    let cases: Vec<(&str, Box<dyn Fn(&OwnedJsValue) -> bool>)> = vec![
        ("undefined", Box::new(|v| v.is_undefined())),
        ("null", Box::new(|v| v.is_null())),
        ("true", Box::new(|v| v.is_bool() && v.to_bool().unwrap())),
        ("false", Box::new(|v| v.is_bool() && !v.to_bool().unwrap())),
        ("2 > 10", Box::new(|v| v.is_bool() && !v.to_bool().unwrap())),
        ("1", Box::new(|v| v.is_int() && v.to_int().unwrap() == 1)),
        (
            "1 + 1",
            Box::new(|v| v.is_int() && v.to_int().unwrap() == 2),
        ),
        (
            "1.1",
            Box::new(|v| v.is_float() && v.to_float().unwrap() == 1.1),
        ),
        (
            "2.2 * 2 + 5",
            Box::new(|v| v.is_float() && v.to_float().unwrap() == 9.4),
        ),
        (
            "\"abc\"",
            Box::new(|v| v.is_string() && v.to_string().unwrap() == "abc"),
        ),
        (
            "[1,2]",
            Box::new(|v| {
                if v.is_array() {
                    let arr = v.to_array().unwrap();
                    if arr.length() == 2
                        && arr.get_index(0).unwrap().unwrap().to_int().unwrap() == 1
                        && arr.get_index(1).unwrap().unwrap().to_int().unwrap() == 2
                    {
                        return true;
                    }
                }

                false
            }),
        ),
    ];

    for (code, res) in cases.into_iter() {
        let v = c.eval(code, false).unwrap();
        assert!(res(&v));
    }

    let obj_cases: Vec<(&str, Box<dyn Fn(&OwnedJsValue) -> bool>)> = vec![
        (
            r#" {"a": null, "b": undefined} "#,
            Box::new(|v| {
                if v.is_object() {
                    let obj = v.clone().try_into_object().unwrap();
                    if obj.property("a").unwrap().unwrap().is_null()
                        && obj.property("b").unwrap().unwrap().is_undefined()
                    {
                        return true;
                    }
                }

                false
            }),
        ),
        (
            r#" {a: 1, b: true, c: {c1: false}} "#,
            Box::new(|v| {
                if v.is_object() {
                    let obj = v.clone().try_into_object().unwrap();
                    if obj.property("a").unwrap().unwrap().to_int().unwrap() == 1
                        && obj.property("b").unwrap().unwrap().to_bool().unwrap()
                        && obj.property("c").unwrap().unwrap().is_object()
                    {
                        let c = obj
                            .property("c")
                            .unwrap()
                            .unwrap()
                            .try_into_object()
                            .unwrap();
                        if !c.property("c1").unwrap().unwrap().to_bool().unwrap() {
                            return true;
                        }
                    }
                }

                false
            }),
        ),
    ];

    for (index, (code, res)) in obj_cases.into_iter().enumerate() {
        let full_code = format!(
            "var v{index} = {code}; v{index}",
            index = index,
            code = code
        );

        let v = c.eval(&full_code, false).unwrap();
        assert!(res(&v));
    }

    assert!(c.eval_as::<bool>("true").unwrap(),);
    assert_eq!(c.eval_as::<i32>("1 + 2").unwrap(), 3,);

    let value: String = c.eval_as("var x = 44; x.toString()").unwrap();
    assert_eq!(&value, "44");

    #[cfg(feature = "bigint")]
    assert_eq!(
        c.eval_as::<num_bigint::BigInt>("1n << 100n").unwrap(),
        num_bigint::BigInt::from(1i128 << 100)
    );

    #[cfg(feature = "bigint")]
    assert_eq!(c.eval_as::<i64>("1 << 30").unwrap(), 1i64 << 30);

    #[cfg(feature = "bigint")]
    assert_eq!(c.eval_as::<u128>("1n << 100n").unwrap(), 1u128 << 100);
}

#[test]
fn test_eval_syntax_error() {
    let c = Context::builder().build().unwrap();
    let ctx = c.context_raw();
    assert_eq!(
        c.eval(
            r#"
            !!!!
        "#,
            false
        ),
        Err(ExecutionError::Exception(owned!(
            ctx,
            "SyntaxError: unexpected token in expression: \'\'"
        )))
    );
}

#[test]
fn test_eval_exception() {
    let c = Context::builder().build().unwrap();
    let ctx = c.context_raw();
    assert_eq!(
        c.eval(
            r#"
            function f() {
                throw new Error("My Error");
            }
            f();
        "#,
            false
        ),
        Err(ExecutionError::Exception(owned!(ctx, "Error: My Error")))
    );
}

#[test]
fn eval_async() {
    let c = Context::builder().build().unwrap();

    let value = c
        .eval(
            r#"
        new Promise((resolve, _) => {
            resolve(33);
        })
    "#,
            true,
        )
        .unwrap();
    assert_eq!(value.to_int().unwrap(), 33,);

    let res = c.eval(
        r#"
        new Promise((_resolve, reject) => {
            reject("Failed...");
        })
    "#,
        true,
    );
    assert!(res.is_err());
    assert!(res.is_err() && res.unwrap_err().to_string().contains("Failed..."),);
}

#[test]
fn test_set_global() {
    let context = Context::builder().build().unwrap();
    context.set_global("someGlobalVariable", 42).unwrap();
    let value = context.eval_as::<i32>("someGlobalVariable").unwrap();
    assert_eq!(value, 42,);
}

#[test]
fn test_call() {
    let c = Context::builder().build().unwrap();
    let ctx = c.context_raw();

    let arg: OwnedJsValue = (ctx, "22").into();

    assert_eq!(
        c.call_function("parseInt", vec![arg])
            .unwrap()
            .to_int()
            .unwrap(),
        22
    );

    c.eval(
        r#"
        function add(a, b) {
            return a + b;
        }
    "#,
        false,
    )
    .unwrap();
    assert_eq!(
        c.call_function("add", vec![5, 7])
            .unwrap()
            .to_int()
            .unwrap(),
        12,
    );

    c.eval(
        r#"
        function sumArray(arr) {
            let sum = 0;
            for (const value of arr) {
                sum += value;
            }
            return sum;
        }
    "#,
        false,
    )
    .unwrap();
    assert_eq!(
        c.call_function("sumArray", vec![owned!(ctx, vec![1, 2, 3])])
            .unwrap()
            .to_int()
            .unwrap(),
        6,
    );

    c.eval(
        r#"
        function addObject(obj) {
            let sum = 0;
            for (const key of Object.keys(obj)) {
                sum += obj[key];
            }
            return sum;
        }
    "#,
        false,
    )
    .unwrap();
    let mut obj = std::collections::HashMap::<String, i32>::new();
    obj.insert("a".to_string(), 10);
    obj.insert("b".to_string(), 20);
    obj.insert("c".to_string(), 30);
    assert_eq!(
        c.call_function("addObject", vec![owned!(ctx, obj)])
            .unwrap()
            .to_int()
            .unwrap(),
        60
    );
}

#[test]
fn test_call_large_string() {
    let c = Context::builder().build().unwrap();
    let ctx = c.context_raw();
    c.eval(" function strLen(s) { return s.length; } ", false)
        .unwrap();

    let s = " ".repeat(200_000);
    let v = c
        .call_function("strLen", vec![owned!(ctx, s)])
        .unwrap()
        .to_int()
        .unwrap();
    assert_eq!(v, 200_000);
}

#[test]
fn call_async() {
    let c = Context::builder().build().unwrap();
    let ctx = c.context_raw();

    c.eval(
        r#"
        function asyncOk() {
            return new Promise((resolve, _) => {
                resolve(33);
            });
        }

        function asyncErr() {
            return new Promise((_resolve, reject) => {
                reject("Failed...");
            });
        }
    "#,
        false,
    )
    .unwrap();

    let value = c
        .call_function("asyncOk", vec![owned!(ctx, true)])
        .unwrap()
        .to_int()
        .unwrap();
    assert_eq!(value, 33);

    let res = c.call_function("asyncErr", vec![owned!(ctx, true)]);
    assert_eq!(
        res,
        Err(ExecutionError::Exception(owned!(ctx, "Failed...")))
    );
}

#[test]
fn test_callback() {
    let c = Context::builder().build().unwrap();

    c.add_callback("no_arguments", || true).unwrap();
    assert!(c.eval_as::<bool>("no_arguments()").unwrap());

    c.add_callback("no_arguments", || false).unwrap();
    assert!(!c.eval_as::<bool>("no_arguments()").unwrap());

    c.add_callback("cb1", |flag: bool| !flag).unwrap();
    assert!(!c.eval("cb1(true)", false).unwrap().to_bool().unwrap(),);

    c.add_callback("concat2", |a: String, b: String| format!("{}{}", a, b))
        .unwrap();
    assert_eq!(
        c.eval(r#"concat2("abc", "def")"#, false)
            .unwrap()
            .to_string()
            .unwrap(),
        "abcdef",
    );

    c.add_callback("add2", |a: i32, b: i32| -> i32 { a + b })
        .unwrap();
    assert_eq!(c.eval("add2(5, 11)", false).unwrap().to_int().unwrap(), 16,);

    c.add_callback("sum", |items: Vec<i32>| -> i32 { items.iter().sum() })
        .unwrap();
    assert_eq!(
        c.eval("sum([1, 2, 3, 4, 5, 6])", false)
            .unwrap()
            .to_int()
            .unwrap(),
        21,
    );

    // c.add_callback("identity", |value: OwnedJsValue| -> OwnedJsValue { value })
    //     .unwrap();
    // {
    //     let v = JsValue::from(22);
    //     assert_eq!(c.eval("identity(22)").unwrap(), v);
    // }
}

#[test]
fn test_callback_argn_variants() {
    macro_rules! callback_argn_tests {
        [
            $(
                $len:literal : ( $( $argn:ident : $argv:literal ),* ),
            )*
        ] => {
            $(
                {
                    // Test plain return type.
                    let name = format!("cb{}", $len);
                    let c = Context::builder().build().unwrap();
                    let ctx = c.context_raw();
                    c.add_callback(&name, | $( $argn : i32 ),*| -> i32 {
                        $( $argn + )* 0
                    }).unwrap();

                    let code = format!("{}( {} )", name, "1,".repeat($len));
                    let v = c.eval(&code,false).unwrap();
                    assert_eq!(v.to_int().unwrap(), $len);

                    // Test Result<T, E> return type with OK(_) returns.
                    let name = format!("cbres{}", $len);
                    c.add_callback(&name, | $( $argn : i32 ),*| -> Result<i32, String> {
                        Ok($( $argn + )* 0)
                    }).unwrap();

                    let code = format!("{}( {} )", name, "1,".repeat($len));
                    let v = c.eval(&code,false).unwrap();
                    assert_eq!(v.to_int().unwrap(), $len);

                    // Test Result<T, E> return type with Err(_) returns.
                    let name = format!("cbreserr{}", $len);
                    c.add_callback(&name, #[allow(unused_variables)] | $( $argn : i32 ),*| -> Result<i32, String> {
                        Err("error".to_string())
                    }).unwrap();

                    let code = format!("{}( {} )", name, "1,".repeat($len));
                    let res = c.eval(&code,false);
                    assert_eq!(res, Err(ExecutionError::Exception(owned!(ctx, "error"))));
                }
            )*
        }
    }

    callback_argn_tests![
        1: (a : 1),
    ]
}

#[test]
fn test_callback_varargs() {
    let c = Context::builder().build().unwrap();

    // No return.
    c.add_callback("cb", |args: Arguments| {
        let args = args.into_vec();
        assert_eq!(args.len(), 3);
        assert_eq!(args.first().unwrap().to_string(), Ok("hello".to_string()));
        assert_eq!(args.get(1).unwrap().to_bool(), Ok(true));
        assert_eq!(args.get(2).unwrap().to_int(), Ok(100));
    })
    .unwrap();
    assert!(c
        .eval_as::<bool>("cb('hello', true, 100) === undefined")
        .unwrap());

    // With return.
    c.add_callback("cb2", |args: Arguments| -> u32 {
        let args = args.into_vec();
        assert_eq!(args.len(), 3);
        assert_eq!(args.first().unwrap().to_int(), Ok(1));
        assert_eq!(args.get(1).unwrap().to_int(), Ok(10));
        assert_eq!(args.get(2).unwrap().to_int(), Ok(100));

        111
    })
    .unwrap();
    c.eval(
        r#"
        var x = cb2(1, 10, 100);
        if (x !== 111) {
        throw new Error('Expected 111, got ' + x);
        }
    "#,
        false,
    )
    .unwrap();
}

#[test]
fn test_callback_invalid_argcount() {
    let c = Context::builder().build().unwrap();
    let ctx = c.context_raw();

    c.add_callback("cb", |a: i32, b: i32| a + b).unwrap();

    assert_eq!(
        c.eval(" cb(5) ", false),
        Err(ExecutionError::Exception(owned!(
            ctx,
            "Invalid argument count: Expected 2, got 1"
        ))),
    );
}

#[test]
fn memory_limit_exceeded() {
    let c = Context::builder().memory_limit(100_000).build().unwrap();
    assert_eq!(
        c.eval("  'abc'.repeat(200_000) ", false),
        Err(ExecutionError::OutOfMemory),
    );
}

#[test]
fn test_create_callback() {
    let context = Context::builder().build().unwrap();
    let ctx = context.context_raw();

    // Register an object.
    let mut obj = HashMap::<String, OwnedJsValue>::new();

    // insert add function into the object.
    obj.insert(
        "add".to_string(),
        owned!(
            ctx,
            context.create_callback(|a: i32, b: i32| a + b).unwrap()
        ),
    );
    context.set_global("myObj", owned!(ctx, obj)).unwrap();

    let output = context.eval_as::<i32>("myObj.add( 3 , 4 ) ").unwrap();

    assert_eq!(output, 7);
}

#[test]
fn context_reset() {
    let c = Context::builder().build().unwrap();
    c.eval(" var x = 123; ", false).unwrap();
    c.add_callback("myCallback", || true).unwrap();

    let c2 = c.reset().unwrap();

    // Check it still works.
    assert_eq!(
        c2.eval_as::<String>(" 'abc'.repeat(2) ").unwrap(),
        "abcabc".to_string(),
    );

    // Check old state is gone.
    let err_msg = c2.eval(" x ", false).unwrap_err().to_string();
    assert!(err_msg.contains("ReferenceError"));

    // Check callback is gone.
    let err_msg = c2.eval(" myCallback() ", false).unwrap_err().to_string();
    assert!(err_msg.contains("ReferenceError"));
}

#[inline(never)]
fn build_context() -> Context {
    let ctx = Context::builder().build().unwrap();
    let name = "cb".to_string();
    ctx.add_callback(&name, |a: String| a.repeat(2)).unwrap();

    let code = " function f(value) { return cb(value); } ".to_string();
    ctx.eval(&code, false).unwrap();

    ctx
}

#[test]
fn moved_context() {
    let c = build_context();
    let ctx = c.context_raw();
    let v = c.call_function("f", vec![owned!(ctx, "test")]).unwrap();
    assert_eq!(v.to_string().unwrap(), "testtest");

    let v = c.eval(" f('la') ", false).unwrap();
    assert_eq!(v.to_string().unwrap(), "lala");
}

#[cfg(feature = "chrono")]
#[test]
fn chrono_serialize() {
    let c = build_context();
    let ctx = c.context_raw();

    c.eval(
        "
        function dateToTimestamp(date) {
            return date.getTime();
        }
    ",
        false,
    )
    .unwrap();

    let now = chrono::Utc::now();
    let now_millis = now.timestamp_millis();

    let timestamp = c
        .call_function("dateToTimestamp", vec![owned!(ctx, now)])
        .unwrap();

    assert_eq!(timestamp.to_float().unwrap(), now_millis as f64);
}

#[cfg(feature = "chrono")]
#[test]
fn chrono_deserialize() {
    use chrono::offset::TimeZone;

    let c = build_context();

    let value = c.eval(" new Date(1234567555) ", false).unwrap();
    let datetime = chrono::Utc.timestamp_millis_opt(1234567555).unwrap();

    assert_eq!(value.to_date().unwrap(), datetime);
}

#[cfg(feature = "chrono")]
#[test]
fn chrono_roundtrip() {
    let c = build_context();
    let ctx = c.context_raw();

    c.eval(" function identity(x) { return x; } ", false)
        .unwrap();
    let d = chrono::Utc::now();
    let td2 = c.call_function("identity", vec![owned!(ctx, d)]).unwrap();
    let d2 = if let Ok(x) = td2.to_date() {
        x
    } else {
        panic!("expected date")
    };

    assert_eq!(d.timestamp_millis(), d2.timestamp_millis());
}

#[cfg(feature = "bigint")]
#[test]
fn test_bigint_deserialize_i64() {
    for i in [0, std::i64::MAX, std::i64::MIN] {
        let c = Context::builder().build().unwrap();
        let value = c.eval(&format!("{}n", i), false).unwrap();
        assert_eq!(value.to_bigint(), Ok(i.into()));
    }
}

#[cfg(feature = "bigint")]
#[test]
fn test_bigint_deserialize_bigint() {
    for i in [
        std::i64::MAX as i128 + 1,
        std::i64::MIN as i128 - 1,
        std::i128::MAX,
        std::i128::MIN,
    ] {
        let c = Context::builder().build().unwrap();
        let value = c.eval(&format!("{}n", i), false).unwrap();
        let expected = num_bigint::BigInt::from(i);
        assert_eq!(value.to_bigint(), Ok(expected.into()));
    }
}

#[cfg(feature = "bigint")]
#[test]
fn test_bigint_serialize_i64() {
    for i in [0, std::i64::MAX, std::i64::MIN] {
        let c = Context::builder().build().unwrap();
        let ctx = c.context_raw();
        c.eval(
            &format!(" function isEqual(x) {{ return x === {}n }} ", i),
            false,
        )
        .unwrap();
        let bigint: BigInt = i.into();
        assert_eq!(
            c.call_function("isEqual", vec![owned!(ctx, bigint)])
                .unwrap()
                .to_bool(),
            Ok(true)
        );
    }
}

#[cfg(feature = "bigint")]
#[test]
fn test_bigint_serialize_bigint() {
    for i in [
        std::i64::MAX as i128 + 1,
        std::i64::MIN as i128 - 1,
        std::i128::MAX,
        std::i128::MIN,
    ] {
        let c = Context::builder().build().unwrap();
        let ctx = c.context_raw();
        c.eval(
            &format!(" function isEqual(x) {{ return x === {}n }} ", i),
            false,
        )
        .unwrap();
        let value: BigInt = num_bigint::BigInt::from(i).into();
        assert_eq!(
            c.call_function("isEqual", vec![owned!(ctx, value)])
                .unwrap()
                .to_bool(),
            Ok(true)
        );
    }
}

#[test]
fn test_console() {
    use console::Level;
    use std::sync::{Arc, Mutex};

    let messages = Arc::new(Mutex::new(Vec::<(Level, Vec<OwnedJsValue>)>::new()));

    let m = messages.clone();
    let c = Context::builder()
        .console(move |level: Level, args: Vec<OwnedJsValue>| {
            m.lock().unwrap().push((level, args));
        })
        .build()
        .unwrap();

    c.eval(
        r#"
        console.log("hi");
        console.error(false);
    "#,
        false,
    )
    .unwrap();

    {
        let m = messages.lock().unwrap();

        assert_eq!(m.len(), 2);
        assert_eq!(m.first().unwrap().0, Level::Log);
        assert_eq!(m.get(1).unwrap().0, Level::Error);
        assert_eq!(m.first().unwrap().1.len(), 1);
        assert_eq!(m.get(1).unwrap().1.len(), 1);
        assert_eq!(
            m.first().unwrap().1.first().unwrap().to_string().unwrap(),
            "hi"
        );
        assert!(!m.get(1).unwrap().1.first().unwrap().to_bool().unwrap());
    }

    // release OwnedJsValue before the Context is dropped,
    // or it will cause a double free error.
    for (level, args) in messages.lock().unwrap().drain(..) {
        println!("{:?} {:?}", level, args);
        drop(args);
    }
}

#[test]
fn test_global_setter() {
    let context = Context::builder().build().unwrap();
    let ctx = context.context_raw();

    context.set_global("a", owned!(ctx, "a")).unwrap();
    context.eval("a + 1", false).unwrap();
}
