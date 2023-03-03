# libquickjs-sys

FFI Bindings for [quickjs](https://bellard.org/quickjs/), a Javascript engine.

See the [quick](https://crates.io/crates/quickjs) crate for a high-level
wrapper.


*Version 0.9.0*
**Embedded VERSION: 2021-03-27**

## Embedded vs system

By default, an embedded version of quickjs is used.

If you want to use a version installed on your system, use:


```toml
libquickjs-sys = { version = "...", default-features = false, features = ["system"] }
```


## Updating the embedded bindings

QuickJS sources and a pre-generated `bindings.rs` are included in the repo.

They are used if the `embedded` feature is enabled.

To updat the bindings, follow these steps:

* (Install [just](https://github.com/casey/just))
* (Install [bindgen-cli](https://rust-lang.github.io/rust-bindgen/command-line-usage.html))
* Update the download URL in ./justfile
* run `just update-quickjs`

Tips:

You may encounter problems in generating bindings.rs like "`FP_SUBNORMAL` redefined here".
[That's the solution](https://github.com/rust-lang/rust-bindgen/issues/687#issuecomment-450750547),
but due that we execute `bindgen` in cli, we have to resolve them by hand.