use quickjspp::serde::to_js;
use quickjspp::Context;
use serde_json::json;

pub fn main() {
    let context = Context::new().unwrap();

    // generate a complex json value (rust format)
    // for simplicity, we use serde_json::json! macro here,
    // please note that it is still a rust format value.
    let value = json!({
        "name": "John Doe",
        "age": 43,
        "phones": [
            "+44 1234567",
            "+44 2345678"
        ]
    });

    // use our to_js function to convert rust value to js value
    // now it is a js value in quickjs context
    let js_value = to_js(context.context_raw(), &value).unwrap();

    // get json string from quickjs's JSON.stringify method
    println!("json output: {}", js_value.to_json_string(2).unwrap());
}
