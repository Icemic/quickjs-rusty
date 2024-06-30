![quickjs-rusty](https://socialify.git.ci/Icemic/quickjs-rusty/image?description=1&language=1&name=1&owner=1&stargazers=1&theme=Light)

[![Crates.io](https://img.shields.io/crates/v/quickjs-rusty.svg?maxAge=3600)](https://crates.io/crates/quickjs-rusty)
[![docs.rs](https://docs.rs/quickjs-rusty/badge.svg)](https://docs.rs/quickjs-rusty)

QuickJS is a small and embeddable Javascript engine by Fabrice Bellard and Charlie Gordon. It supports the ES2023 specification including modules, asynchronous generators, proxies and BigInt.  
Quickjs-NG is one of the most active forks of QuickJS, and it is maintained by the community focused on reigniting the project.

This crate allows you to easily access and use all the features of QuickJS from Rust. It also provides robust Rust-JS type conversion and interoperability capabilities.

## Quickstart

```toml
[dependencies]
quickjs-rusty = "0.6.1"
```

```rust
use quickjs_rusty::{Context, JsValue};

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

Note: This project is derived from [quickjs-rs](https://github.com/theduke/quickjs-rs), but it has undergone significant restructuring. It features a completely different code structure and functional design compared to the original project.

## Optional Features

The crate supports the following features:

- `serde`: _(default enabled)._ enable serde method `from_js` and `to_js` to transform between Rust types and js value in quickjs context. It should compatible with `serde_json` but not tested yet. See more on the [example](/examples/serde.rs).
- `chrono`: _(default enabled)._ chrono integration
  - adds a `JsValue::Date` variant that can be (de)serialized to/from a JS `Date`
- `bigint`: _(default enabled)._ arbitrary precision integer support via [num-bigint](https://github.com/rust-num/num-bigint)

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
