use quickjspp::serde::to_js;
use quickjspp::Context;

#[test]
fn serde_bool() {
    let context = Context::new().unwrap();
    // bool
    let value = true;
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "true");
}

#[test]
fn serde_int() {
    let context = Context::new().unwrap();
    // int
    // TODO: should take care of i32, i64, u32, u64, etc.
    let value = 123;
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "123");
}

#[test]
fn serde_float() {
    let context = Context::new().unwrap();
    // float
    let value = 3.1415;
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "3.1415");
}

#[test]
fn serde_char() {
    let context = Context::new().unwrap();
    // char
    let value = 'a';
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "\"a\"");
}

#[test]
fn serde_string() {
    let context = Context::new().unwrap();
    // string
    let value = "哈哈";
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "\"哈哈\"");
}

#[test]
fn serde_null_none() {
    let context = Context::new().unwrap();
    // null (None)
    let value: Option<bool> = None;
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "null");
}

#[test]
fn serde_null_unit_struct() {
    let context = Context::new().unwrap();
    // null (unit struct)
    let value = SimpleUnitStruct;
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "null");
}

#[test]
fn serde_unit_variant() {
    let context = Context::new().unwrap();
    // unit variant
    let value = SimpleEnum::A;
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "\"A\"");
}

#[test]
fn serde_newtype_variant() {
    let context = Context::new().unwrap();
    // newtype variant
    let value = SimpleEnum::Foo("bar".to_string());
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "{\"Foo\":\"bar\"}");
}

#[test]
fn serde_newtype_variant_tuple() {
    let context = Context::new().unwrap();
    // newtype variant tuple
    let value = SimpleEnum::D(true, 2233);
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "{\"D\":[true,2233]}");
}

#[test]
fn serde_newtype_variant_tuple_empty() {
    let context = Context::new().unwrap();
    // newtype variant tuple empty
    let value = SimpleEnum::B();
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "{\"B\":[]}");
}

#[test]
fn serde_newtype_variant_struct() {
    let context = Context::new().unwrap();
    // newtype variant struct
    let value = SimpleEnum::C {
        a: 233,
        foo: "Bar".to_string(),
    };
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(
        js_value.to_json_string(0).unwrap(),
        "{\"C\":{\"a\":233,\"foo\":\"Bar\"}}"
    );
}

#[test]
fn serde_newtype_struct() {
    let context = Context::new().unwrap();
    // newtype struct
    let value = SimpleNewTypeStruct(100);
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "100");
}

#[test]
fn serde_tuple_struct() {
    let context = Context::new().unwrap();
    // tuple struct
    let value = SimpleTupleStruct(100, 101);
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "[100,101]");
}

#[test]
fn serde_struct() {
    let context = Context::new().unwrap();
    // simple struct
    let value = SimpleStruct { a: 100, b: 101 };
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "{\"a\":100,\"b\":101}");
}

#[test]
fn serde_vector() {
    let context = Context::new().unwrap();
    // vector
    let value = vec![1, 2, 3, 4, 5];
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "[1,2,3,4,5]");
}

#[test]
fn serde_tuple() {
    let context = Context::new().unwrap();
    // tuple
    let value = (123, 3.14, "hh");
    let js_value = to_js(context.context_raw(), &value).unwrap();

    assert_eq!(js_value.to_json_string(0).unwrap(), "[123,3.14,\"hh\"]");
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct SimpleUnitStruct;
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct SimpleNewTypeStruct(i32);
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct SimpleTupleStruct(i32, i32);
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct SimpleStruct {
    a: i32,
    b: i32,
}

type Bar = String;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
enum SimpleEnum {
    A,
    B(),
    C { a: i32, foo: String },
    D(bool, u32),
    Foo(Bar),
}
