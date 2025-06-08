# libquickjs-sys

FFI Bindings for [quickjs-ng](https://github.com/quickjs-ng/quickjs), a fork of [quickjs](https://bellard.org/quickjs/), which is a Javascript engine.

See the [quickjs-rusty](https://crates.io/crates/quickjs-rusty) crate for a high-level
wrapper.

## Updating the embedded bindings

QuickJS sources and a pre-generated `bindings.rs` are included in the repo.

They are used if the `embedded` feature is enabled.

To update the bindings, follow these steps:

* (Install [just](https://github.com/casey/just))
* (Install [bindgen-cli](https://rust-lang.github.io/rust-bindgen/command-line-usage.html))
* Update the download URL in ./justfile
* run `just update-quickjs`

Tips:

You may encounter problems in generating bindings.rs like "`FP_SUBNORMAL` redefined here".
[That's the solution](https://github.com/rust-lang/rust-bindgen/issues/687#issuecomment-450750547),
but due that we execute `bindgen` in cli, we have to resolve them by hand.
