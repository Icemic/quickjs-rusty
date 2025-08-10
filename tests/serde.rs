use std::fmt::Debug;

use libquickjs_ng_sys::JSContext;
use quickjs_rusty::serde::{from_js, to_js};
use quickjs_rusty::{value::OwnedJsValue, Context};
use serde_json::{json, Value};

#[test]
fn serde_ser_bool() {
    let context = Context::builder().build().unwrap();
    // bool
    let value = true;
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "true");
}

#[test]
fn serde_ser_int() {
    let context = Context::builder().build().unwrap();
    // int
    // TODO: should take care of i32, i64, u32, u64, etc.
    let value = 123;
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "123");
}

#[test]
fn serde_ser_int64() {
    let context = Context::builder().build().unwrap();
    // int
    // TODO: should take care of i32, i64, u32, u64, etc.
    let value: u64 = 1754784747637;
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "1754784747637");
}

#[test]
fn serde_ser_float() {
    let context = Context::builder().build().unwrap();
    // float
    let value = 3.1415;
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "3.1415");
}

#[test]
fn serde_ser_char() {
    let context = Context::builder().build().unwrap();
    // char
    let value = 'a';
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "\"a\"");
}

#[test]
fn serde_ser_string() {
    let context = Context::builder().build().unwrap();
    // string
    let value = "哈哈";
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "\"哈哈\"");
}

#[test]
fn serde_ser_null_none() {
    let context = Context::builder().build().unwrap();
    // null (None)
    let value: Option<bool> = None;
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "null");
}

#[test]
fn serde_ser_null_unit_struct() {
    let context = Context::builder().build().unwrap();
    // null (unit struct)
    let value = SimpleUnitStruct;
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "null");
}

#[test]
fn serde_ser_some() {
    let context = Context::builder().build().unwrap();
    // null (None)
    let value: Option<bool> = Some(true);
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "true");
}

#[test]
fn serde_ser_unit_variant() {
    let context = Context::builder().build().unwrap();
    // unit variant
    let value = SimpleEnum::A;
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "\"A\"");
}

#[test]
fn serde_ser_newtype_variant() {
    let context = Context::builder().build().unwrap();
    // newtype variant
    let value = SimpleEnum::Foo("bar".to_string());
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "{\"Foo\":\"bar\"}");
}

#[test]
fn serde_ser_newtype_variant_tuple() {
    let context = Context::builder().build().unwrap();
    // newtype variant tuple
    let value = SimpleEnum::D(true, 2233);
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "{\"D\":[true,2233]}");
}

#[test]
fn serde_ser_newtype_variant_tuple_empty() {
    let context = Context::builder().build().unwrap();
    // newtype variant tuple empty
    let value = SimpleEnum::B();
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "{\"B\":[]}");
}

#[test]
fn serde_ser_newtype_variant_struct() {
    let context = Context::builder().build().unwrap();
    // newtype variant struct
    let value = SimpleEnum::C {
        a: 233,
        foo: "Bar".to_string(),
    };
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(
        js_value.to_json_string(0).unwrap(),
        "{\"C\":{\"a\":233,\"foo\":\"Bar\"}}"
    );
}

#[test]
fn serde_ser_newtype_struct() {
    let context = Context::builder().build().unwrap();
    // newtype struct
    let value = SimpleNewTypeStruct(100);
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "100");
}

#[test]
fn serde_ser_tuple_struct() {
    let context = Context::builder().build().unwrap();
    // tuple struct
    let value = SimpleTupleStruct(100, 101);
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "[100,101]");
}

#[test]
fn serde_ser_struct() {
    let context = Context::builder().build().unwrap();
    // simple struct
    let value = SimpleStruct { a: 100, b: 101 };
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "{\"a\":100,\"b\":101}");
}

#[test]
fn serde_ser_vector() {
    let context = Context::builder().build().unwrap();
    // vector
    let value = vec![1, 2, 3, 4, 5];
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "[1,2,3,4,5]");
}

#[test]
fn serde_ser_tuple() {
    let context = Context::builder().build().unwrap();
    // tuple
    let value = (123, 3.14, "hh");
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "[123,3.14,\"hh\"]");
}

#[test]
fn serde_ser_map() {
    let context = Context::builder().build().unwrap();
    // map
    let mut value = std::collections::HashMap::new();
    value.insert("a".to_string(), 1);
    value.insert("b".to_string(), 2);
    value.insert("c".to_string(), 3);
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    let json_str = js_value.to_json_string(0).unwrap();

    // the order of keys is not stable, so we just check the content
    assert!(json_str.contains("\"a\":1"));
    assert!(json_str.contains("\"b\":2"));
    assert!(json_str.contains("\"c\":3"));
}

fn parse_from_js<T: serde::de::DeserializeOwned>(value: Value) -> T {
    let context = Context::builder().build().unwrap();
    // use our to_js function to convert rust value to js value
    // now it is a js value in quickjs context
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    match from_js::<T>(unsafe { context.context_raw() }, &js_value) {
        Ok(v) => v,
        Err(err) => {
            panic!("{}", err);
        }
    }
}

fn parse_from_js_borrowed<'a, T: serde::de::Deserialize<'a>>(
    context: *mut JSContext,
    value: &'a OwnedJsValue,
) -> T {
    match from_js::<T>(context, value) {
        Ok(v) => v,
        Err(err) => {
            panic!("{}", err);
        }
    }
}

#[test]
fn serde_de_bool() {
    // generate a complex json value (rust format)
    // for simplicity, we use serde_json::json! macro here,
    // please note that it is still a rust format value.
    let value = json!(true);
    assert!(parse_from_js::<bool>(value));
}

#[test]
fn serde_de_unsigned_interger() {
    let value = json!(1234);
    assert_eq!(parse_from_js::<u32>(value), 1234);
}

#[test]
fn serde_de_signed_interger() {
    let value = json!(-1234);
    assert_eq!(parse_from_js::<i32>(value), -1234);
}

#[test]
fn serde_de_i64() {
    let value = json!(1754784747637 as i64);

    // number larger than i32::MAX is treated as f64 in quickjs
    assert_eq!(parse_from_js::<f64>(value), 1754784747637.0);
}

#[test]
fn serde_de_float() {
    let value = json!(3.14159265);
    assert_eq!(parse_from_js::<f64>(value), 3.14159265);
}

#[test]
fn serde_de_option_none() {
    let value = json!(None::<()>);
    assert_eq!(parse_from_js::<Option<()>>(value), None::<()>);
}

#[test]
fn serde_de_option_some_with_value() {
    let value = json!(Some(true));
    assert_eq!(parse_from_js::<Option<bool>>(value), Some(true));
}

#[test]
fn serde_de_option_some_with_value2() {
    let value = json!(Some(123));
    assert_eq!(parse_from_js::<Option<i32>>(value), Some(123));
}

// in json, Some(()) is the same as None
#[test]
fn serde_de_option_some() {
    let value = json!(Some(()));
    assert_eq!(parse_from_js::<Option<()>>(value), None);
}

#[test]
fn serde_de_string() {
    let value = json!("😄");
    assert_eq!(parse_from_js::<String>(value), "😄");
}

#[test]
fn serde_de_borrowed_str() {
    let context = Context::builder().build().unwrap();
    let value = json!("😄");
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();
    assert_eq!(
        parse_from_js_borrowed::<&str>(unsafe { context.context_raw() }, &js_value),
        "😄"
    );
}

#[test]
fn serde_de_char() {
    let value = json!("😄");
    assert_eq!(parse_from_js::<char>(value), '😄');
}

#[test]
fn serde_de_unit_struct() {
    let value = json!(null);
    assert_eq!(parse_from_js::<SimpleUnitStruct>(value), SimpleUnitStruct);
}

#[test]
fn serde_de_array() {
    let value = json!([1, 2, 3]);
    assert_eq!(parse_from_js::<Vec<u8>>(value), vec![1, 2, 3]);
}

// #[test]
// fn serde_de_borrow_bytes() {
//     let value = json!([1, 2, 3]);
//     assert_eq!(
//         parse_from_js::<&[u8]>(value),
//         vec![1, 2, 3]
//     );
// }

#[test]
fn serde_de_tuple_fixed_vec() {
    let value = json!([1, 2, 3]);
    assert_eq!(parse_from_js::<[u8; 3]>(value), [1, 2, 3]);
}

#[test]
fn serde_de_tuple() {
    let value = json!([100, 101]);
    assert_eq!(parse_from_js::<(u32, u32)>(value), (100, 101));
}

#[test]
fn serde_de_tuple_struct() {
    let value = json!([100, 101]);
    assert_eq!(
        parse_from_js::<SimpleTupleStruct>(value),
        SimpleTupleStruct(100, 101)
    );
}

#[test]
fn serde_de_newtype_struct() {
    let value = json!(SimpleNewTypeStruct(123));
    assert_eq!(
        parse_from_js::<SimpleNewTypeStruct>(value),
        SimpleNewTypeStruct(123)
    );
}

#[test]
fn serde_de_struct() {
    let value = json!(SimpleStruct { a: 123, b: 456 });
    assert_eq!(
        parse_from_js::<SimpleStruct>(value),
        SimpleStruct { a: 123, b: 456 }
    );
}

#[test]
fn serde_de_unit_variant() {
    let value = json!(SimpleEnum::A);
    assert_eq!(parse_from_js::<SimpleEnum>(value), SimpleEnum::A);
}

#[test]
fn serde_de_newtype_variant_empty() {
    let value = json!(SimpleEnum::B());
    assert_eq!(parse_from_js::<SimpleEnum>(value), SimpleEnum::B());
}

#[test]
fn serde_de_newtype_variant_tuple() {
    let value = json!(SimpleEnum::D(false, 222));
    assert_eq!(
        parse_from_js::<SimpleEnum>(value),
        SimpleEnum::D(false, 222)
    );
}

#[test]
fn serde_de_newtype_variant_struct() {
    let value = json!(SimpleEnum::C {
        a: 123,
        foo: "哈哈".to_string()
    });
    assert_eq!(
        parse_from_js::<SimpleEnum>(value),
        SimpleEnum::C {
            a: 123,
            foo: "哈哈".to_string()
        }
    );
}

#[test]
fn serde_de_newtype_variant() {
    let value = json!(SimpleEnum::Foo("哈哈".to_string()));
    assert_eq!(
        parse_from_js::<SimpleEnum>(value),
        SimpleEnum::Foo("哈哈".to_string())
    );
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
struct SimpleUnitStruct;
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
struct SimpleNewTypeStruct(i32);
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
struct SimpleTupleStruct(i32, i32);
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
struct SimpleStruct {
    a: i32,
    b: i32,
}

type Bar = String;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
enum SimpleEnum {
    A,
    B(),
    C { a: i32, foo: String },
    D(bool, u32),
    Foo(Bar),
}
