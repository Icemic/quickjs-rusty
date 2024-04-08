# quickjspp-rs

[![Crates.io](https://img.shields.io/crates/v/quick-js.svg?maxAge=3600)](https://crates.io/crates/quickjspp)
[![docs.rs](https://docs.rs/quick-js/badge.svg)](https://docs.rs/quickjspp)

**This is a fork of [quickjs-rs](https://github.com/theduke/quickjs-rs) but replaces the binding to the original [quickjs](https://bellard.org/quickjs/) by Fabrice Bellard with its fork [quickjspp](https://github.com/c-smile/quickjspp) by Andrew Fedoniouk, which is MSVC compatible/compileable.**

QuickJS is a new, small Javascript engine by Fabrice Bellard and Charlie Gordon.
It is fast and supports the full ES2020 specification.

QuickJSpp is a fork of Quickjs By Andrew Fedoniouk (a.k.a. c-smile).

This crate allows you to easily run and integrate with Javascript code from Rust.

## Quickstart

```toml
[dependencies]
quickjspp = "0.5.0"
```

```rust
use quickjspp::{Context, JsValue};

let context = Context::new().unwrap();

// Eval.

let value = context.eval("1 + 2").unwrap();
assert_eq!(value, JsValue::Int(3));

let value = context.eval_as::<String>(" var x = 100 + 250; x.toString() ").unwrap();
assert_eq!(&value, "350");

// Callbacks.

context.add_callback("myCallback", |a: i32, b: i32| a + b).unwrap();

context.eval(r#"
    // x will equal 30
    var x = myCallback(10, 20);
"#).unwrap();
```

## Optional Features

The crate supports the following features:

- `serde`: _(default enabled)._ enable serde method `from_js` and `to_js` to transform between Rust types and js value in quickjs context. It should compatible with `serde_json` but not tested yet. See more on the [example](/examples/serde.rs).
- `chrono`: chrono integration
  - adds a `JsValue::Date` variant that can be (de)serialized to/from a JS `Date`
- `bigint`: arbitrary precision integer support via [num-bigint](https://github.com/rust-num/num-bigint)
- `log`: allows forwarding `console.log` messages to the `log` crate.
  Note: must be enabled with `ContextBuilder::console(quickjspp::console::LogConsole);`

- `patched`
  Enabled automatically for some other features, like `bigint`.
  You should not need to enable this manually.
  Applies QuickJS patches that can be found in `libquickjs-sys/embed/patches` directory.

## Installation

By default, quickjs is **bundled** with the `libquickjs-sys` crate and
automatically compiled, assuming you have the appropriate dependencies.

### Windows Support

quickjspp-rs can be used under target `x86_64-pc-windows-msvc`, 

### System installation

To use the system installation, without the bundled feature, first install the required
dependencies, and then compile and install quickjspp.

You then need to disable the `bundled` feature in the `libquickjs-sys` crate to
force using the system version.
