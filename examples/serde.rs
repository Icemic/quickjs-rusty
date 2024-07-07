use std::fmt::Debug;

use libquickjs_ng_sys::JSContext;
use quickjs_rusty::serde::{from_js, to_js};
use quickjs_rusty::{Context, OwnedJsValue};
use serde_json::json;

pub fn main() {
    let context = Context::builder().build().unwrap();

    // generate a complex json value (rust format)
    // for simplicity, we use serde_json::json! macro here,
    // please note that it is still a rust format value.
    let value = json!({
        "name": "John Doe",
        "status": "unemployed",
        "age": 43,
        "phones": [
            "+44 1234567",
            "+44 2345678"
        ]
    });

    // use our to_js function to convert rust value to js value
    // now it is a js value in quickjs context
    let js_value = to_js(unsafe { context.context_raw() }, &value).unwrap();

    // get json string from quickjs's JSON.stringify method
    println!("json output: {}", js_value.to_json_string(2).unwrap());

    // parse json back to rust type
    let ret = parse_from_js::<Person>(unsafe { context.context_raw() }, &js_value);
    println!("rust type output: {:#?}", ret);
}

fn parse_from_js<'a, T: serde::de::Deserialize<'a> + Debug>(
    context: *mut JSContext,
    value: &'a OwnedJsValue,
) -> T {
    match from_js::<T>(context, value) {
        Ok(v) => v,
        Err(err) => panic!("{}", err),
    }
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct Person<'a> {
    name: &'a str,
    status: Status,
    age: u8,
    phones: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Employed,
    Unemployed,
}
