# quick-js - Changelog

**ATTENTION: Latest changelog now maintained at Github release page**

# v0.4.3 - 2023-08-16

* feat: add symbol support in JsValue enums
* fix: serde cannot handles symbol
* fix: serde checks circular reference on deserialize

# v0.4.2 - 2023-08-09

* fix: Deserializer treat `undefined` as `Some` wrongly

# v0.4.1 - 2023-08-09

- fix: remove `OwnedValueRef` struct, which duplicate with OwnedJsValue
- fix: serde `deserialize_any` problem
- feat: expose more methods and support custom raw CFunction

# v0.4.0 - 2023-08-06

- support serde

# v0.3.0 - 2023-07-26

- feat: add JsValue::Function enum, and JsFunction can be used as callback parameter or return

# v0.2.0 - 2023-06-23

- add methods to run module codes
- fix: module loader thread safe problem
- refactor: decoupling some methods or structs with ContextWrapper
- fix: module loader exception handling

# v0.1.0 - 2023-03-05

- switch to quickjspp

## Below are the logs from quickjs-rs before fork.

## Master branch

- Fixed use after free in `set_global` [#105](https://github.com/theduke/quickjs-rs/issues/105)
- `add_callback` can now take `JsValue` arguments [#109](https://github.com/theduke/quickjs-rs/issues/109)
- Enable chrono feature by default
- Update to QuickJS 2021-03-27

## v0.4.0 - 2021-02-05

- Bumped quickjs to `2020-11-08`
- Added `Context::set_global`
- Added `JsValue::Undefined` (breaking change)

## v0.3.4 - 2020-07-09

- Bump quickjs to 2020-07-05

## v0.3.3 - 2020-05-27

- Add Windows support
  (only with MSYS2 environment and `x86_64-pc-windows-gnu` target architecture)

## v0.3.2 - 2020-05-25

- Updated quickjs to 2020-04-12

## v0.3.1 - 2020-03-24

- Update quickjs to 2020-03-16
- Add `TryFrom<JsValue>` impl for `HashMap<String, X>`

## v0.3.0 - 2019-11-02

### Features

- Add BigInt integration
- Add logging system and optional `log` crate integration
- Upgrade quickjs to 2019-10-27

### Breaking Changes

- Made `Value` enum non-exhaustive
- new Value::BigInt variant (with `bigint` feature)

## v0.2.3 - 2019-08-30

- Properly free property keys after enumeration
  (Fixes memory leak when deserializing objects)

## v0.2.2 - 2019-08-13

- Fix invalid millisecond conversion for JsValue::Date

## v0.2.1 - 2019-08-13

- Impelemented deserializiation of objects to `JsValue::Object`
- Added `chrono` integration via the `chrono` feature
  Adds a `JsValue::Date(DateTime<Utc>)` variant that allows (de)serializing
  a JS `Date`
- Implemented resolving promises in `eval`/`call_function`
- Added `patched` feature for applying quickjs fixes
- quickjs upgraded to `2019-08-10` release

## v0.2.0 - 2019-07-31

- Added `memory_limit` customization
- Added `Context::clear` method for resetting context
- Callbacks now support more function signatures
  ( up to 5 arguments, `Result<T, E>` return value)
- Updated embedded quickjs bindings to version 2019-07-28.
- Fixed a bug in callback logic
