use std::collections::HashMap;

use quickjs_rusty::BigInt;
use quickjs_rusty::{value::OwnedJsValue, Context};

#[test]
fn test_try_from_owned_js_value() {
    let context = Context::builder().build().unwrap();
    let js_value: OwnedJsValue = context.eval("42", false).unwrap();
    let value: i32 = js_value.try_into().unwrap();
    assert_eq!(value, 42);

    let js_value: OwnedJsValue = context.eval("null", false).unwrap();
    let value: Option<i32> = js_value.try_into().unwrap();
    assert_eq!(value, None);

    let js_value: OwnedJsValue = context.eval("42", false).unwrap();
    let value: Option<i32> = js_value.try_into().unwrap();
    assert_eq!(value, Some(42));

    let js_value: OwnedJsValue = context.eval("1754784747637", false).unwrap();
    let value: Option<u64> = js_value.try_into().unwrap();
    assert_eq!(value, Some(1754784747637));

    let js_value: OwnedJsValue = context.eval("true", false).unwrap();
    let value: bool = js_value.try_into().unwrap();
    assert_eq!(value, true);

    let js_value: OwnedJsValue = context.eval(r#""hello""#, false).unwrap();
    let value: String = js_value.try_into().unwrap();
    assert_eq!(value, "hello");

    let js_value: OwnedJsValue = context.eval(r#"({"key": "value"})"#, false).unwrap();
    let value: HashMap<String, String> = js_value.try_into().unwrap();
    assert_eq!(value, HashMap::from([("key".into(), "value".into())]));

    let js_value: OwnedJsValue = context.eval(r#"[1, 2, 3]"#, false).unwrap();
    let value: Vec<i32> = js_value.try_into().unwrap();
    assert_eq!(value, vec![1, 2, 3]);

    let js_value: OwnedJsValue = context.eval(r#"12345678901234567890n"#, false).unwrap();
    let value: BigInt = js_value.try_into().unwrap();
    assert_eq!(value.to_string(), "12345678901234567890");
}
