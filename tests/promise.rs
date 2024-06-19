use quickjs_rusty::value::*;
use quickjs_rusty::*;

#[test]
fn test_promise_noresolve() {
    let context = Context::builder().build().unwrap();

    let value = context
        .eval("(() => Promise.resolve(123))()", false)
        .unwrap();

    assert_eq!(value.is_object(), true);
    assert_eq!(value.is_promise(), true);
}

#[test]
fn test_promise_resolve() {
    let context = Context::builder().build().unwrap();

    let value = context
        .eval("(() => Promise.resolve(123))()", true)
        .unwrap();

    assert_eq!(value.is_int(), true);
    assert_eq!(value.is_promise(), false);
}

#[test]
fn test_promise_complex() {
    let context = Context::builder().build().unwrap();

    let value = context.eval("() => Promise.resolve(123)", false).unwrap();

    let (promise, resolve, _) = OwnedJsPromise::with_resolvers(&context).unwrap();

    let promise = promise.then(&value).unwrap();

    let on_fulfilled = context
        .create_callback(|aaa: i32| aaa.to_string() + " fulfilled")
        .unwrap();

    let on_fulfilled2 = context
        .eval("(s) => { throw Error(s + ' reject!!!1') }", false)
        .unwrap();

    let on_rejected = context
        .eval("(err) => { return err + 'abc' }", false)
        .unwrap();

    let promise = promise.then(&on_fulfilled).unwrap();
    let promise = promise.then(&on_fulfilled2).unwrap();

    let promise = OwnedJsPromise::all(&context, vec![promise]).unwrap();

    let promise = promise.catch(&on_rejected).unwrap();

    let _ = resolve.call(vec![]).unwrap();

    let _ = context.execute_pending_job();

    assert_eq!(
        promise.result().to_string().unwrap(),
        "Error: 123 fulfilled reject!!!1abc"
    );
}
