![quickjs-rusty](https://socialify.git.ci/Icemic/quickjs-rusty/image?description=1&language=1&name=1&owner=1&stargazers=1&theme=Light)

[![Crates.io](https://img.shields.io/crates/v/quickjs-rusty.svg?maxAge=3600)](https://crates.io/crates/quickjs-rusty)
[![docs.rs](https://docs.rs/quickjs-rusty/badge.svg)](https://docs.rs/quickjs-rusty)

QuickJS is a small and embeddable Javascript engine by Fabrice Bellard and Charlie Gordon. It supports the ES2023 specification including modules, asynchronous generators, proxies and BigInt.  
[Quickjs-ng](https://github.com/quickjs-ng/quickjs) is one of the most active forks of QuickJS, and it is maintained by the community focused on reigniting the project.

This crate allows you to easily access and use all the features of QuickJS from Rust. It also provides robust Rust-JS type conversion and interoperability capabilities.

## Quickstart

```bash
cargo add quickjs-rusty
```

```rust
use quickjs_rusty::{Context, JsValue};

let context = Context::new().unwrap();

// Eval.

let value: String = c.eval_as("var x = 44; x.toString()").unwrap();
assert_eq!(&value, "44");

let value = context.eval_as::<u32>("1 + 2").unwrap();
assert_eq!(&value, 3);

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

We have a builtin quickjs-ng submodule, so you don't need to install quickjs-ng manually. We'll be at best effort to keep the submodule up-to-date with the latest quickjs-ng version.

Make sure you have `Clang` installed on your system.

### Windows Support

quickjs-rusty can be used under target `x86_64-pc-windows-msvc` with `Clang` installed.
