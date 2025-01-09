use std::ops::Deref;

use libquickjs_ng_sys as q;

use crate::utils::ensure_no_excpetion;
use crate::{Context, ExecutionError, JsFunction, ValueError};

use super::OwnedJsValue;

#[derive(Debug, Clone, PartialEq)]
pub struct OwnedJsPromise {
    value: OwnedJsValue,
}

impl OwnedJsPromise {
    pub fn try_from_value(value: OwnedJsValue) -> Result<Self, ValueError> {
        if !value.is_promise() {
            Err(ValueError::Internal("Expected an promise".into()))
        } else {
            Ok(Self { value })
        }
    }

    pub fn into_value(self) -> OwnedJsValue {
        self.value
    }

    pub fn state(&self) -> PromiseState {
        let state = unsafe { q::JS_PromiseState(self.value.context(), self.value.value) };
        match state {
            q::JSPromiseStateEnum_JS_PROMISE_PENDING => PromiseState::Pending,
            q::JSPromiseStateEnum_JS_PROMISE_FULFILLED => PromiseState::Fulfilled,
            q::JSPromiseStateEnum_JS_PROMISE_REJECTED => PromiseState::Rejected,
            _ => unreachable!(),
        }
    }

    /// Returns the result of the promise if the promise's state is in the FULFILLED or REJECTED state,
    /// otherwise returns Undefined.
    pub fn result(&self) -> OwnedJsValue {
        let result = unsafe { q::JS_PromiseResult(self.value.context(), self.value.value) };
        OwnedJsValue::new(self.value.context(), result)
    }

    pub fn then(&self, on_fulfilled: &OwnedJsValue) -> Result<OwnedJsPromise, ExecutionError> {
        let new_promise = unsafe {
            q::JS_Ext_PromiseThen(self.value.context(), self.value.value, on_fulfilled.value)
        };

        let new_promise = OwnedJsValue::new(self.value.context(), new_promise);

        ensure_no_excpetion(self.value.context())?;

        Ok(OwnedJsPromise::try_from_value(new_promise)?)
    }

    pub fn then2(
        &self,
        on_fulfilled: &OwnedJsValue,
        on_rejected: &OwnedJsValue,
    ) -> Result<OwnedJsPromise, ExecutionError> {
        let new_promise = unsafe {
            q::JS_Ext_PromiseThen2(
                self.value.context(),
                self.value.value,
                on_fulfilled.value,
                on_rejected.value,
            )
        };

        let new_promise = OwnedJsValue::new(self.value.context(), new_promise);

        ensure_no_excpetion(self.value.context())?;

        Ok(OwnedJsPromise::try_from_value(new_promise)?)
    }

    pub fn catch(&self, on_rejected: &OwnedJsValue) -> Result<OwnedJsPromise, ExecutionError> {
        let new_promise = unsafe {
            q::JS_Ext_PromiseCatch(self.value.context(), self.value.value, on_rejected.value)
        };

        let new_promise = OwnedJsValue::new(self.value.context(), new_promise);

        ensure_no_excpetion(self.value.context())?;

        Ok(OwnedJsPromise::try_from_value(new_promise)?)
    }

    pub fn finally(&self, on_finally: &OwnedJsValue) -> Result<OwnedJsPromise, ExecutionError> {
        let new_promise = unsafe {
            q::JS_Ext_PromiseFinally(self.value.context(), self.value.value, on_finally.value)
        };

        let new_promise = OwnedJsValue::new(self.value.context(), new_promise);

        ensure_no_excpetion(self.value.context())?;

        Ok(OwnedJsPromise::try_from_value(new_promise)?)
    }

    pub fn resolve(
        context: &Context,
        value: &OwnedJsValue,
    ) -> Result<OwnedJsPromise, ExecutionError> {
        let promise = unsafe { q::JS_Ext_PromiseResolve(context.context, value.value) };
        let promise = OwnedJsValue::new(context.context, promise);

        ensure_no_excpetion(context.context)?;

        Ok(OwnedJsPromise::try_from_value(promise)?)
    }

    pub fn reject(
        context: &Context,
        value: &OwnedJsValue,
    ) -> Result<OwnedJsPromise, ExecutionError> {
        let promise = unsafe { q::JS_Ext_PromiseReject(context.context, value.value) };
        let promise = OwnedJsValue::new(context.context, promise);

        ensure_no_excpetion(context.context)?;

        Ok(OwnedJsPromise::try_from_value(promise)?)
    }

    pub fn all(
        context: &Context,
        values: impl IntoIterator<Item = OwnedJsPromise>,
    ) -> Result<OwnedJsPromise, ExecutionError> {
        let iterable: OwnedJsValue =
            (context.context, values.into_iter().collect::<Vec<_>>()).into();

        let promise = unsafe { q::JS_Ext_PromiseAll(context.context, iterable.value) };
        let promise = OwnedJsValue::new(context.context, promise);

        ensure_no_excpetion(context.context)?;

        Ok(OwnedJsPromise::try_from_value(promise)?)
    }

    pub fn all_settled(
        context: &Context,
        values: impl IntoIterator<Item = OwnedJsPromise>,
    ) -> Result<OwnedJsPromise, ExecutionError> {
        let iterable: OwnedJsValue =
            (context.context, values.into_iter().collect::<Vec<_>>()).into();

        let promise = unsafe { q::JS_Ext_PromiseAllSettled(context.context, iterable.value) };
        let promise = OwnedJsValue::new(context.context, promise);

        ensure_no_excpetion(context.context)?;

        Ok(OwnedJsPromise::try_from_value(promise)?)
    }

    pub fn race(
        context: &Context,
        values: impl IntoIterator<Item = OwnedJsPromise>,
    ) -> Result<OwnedJsPromise, ExecutionError> {
        let iterable: OwnedJsValue =
            (context.context, values.into_iter().collect::<Vec<_>>()).into();

        let promise = unsafe { q::JS_Ext_PromiseRace(context.context, iterable.value) };
        let promise = OwnedJsValue::new(context.context, promise);

        ensure_no_excpetion(context.context)?;

        Ok(OwnedJsPromise::try_from_value(promise)?)
    }

    pub fn any(
        context: &Context,
        values: impl IntoIterator<Item = OwnedJsPromise>,
    ) -> Result<OwnedJsPromise, ExecutionError> {
        let iterable: OwnedJsValue =
            (context.context, values.into_iter().collect::<Vec<_>>()).into();

        let promise = unsafe { q::JS_Ext_PromiseAny(context.context, iterable.value) };
        let promise = OwnedJsValue::new(context.context, promise);

        ensure_no_excpetion(context.context)?;

        Ok(OwnedJsPromise::try_from_value(promise)?)
    }

    pub fn with_resolvers(
        context: &Context,
    ) -> Result<(OwnedJsPromise, JsFunction, JsFunction), ExecutionError> {
        let obj = unsafe { q::JS_Ext_PromiseWithResolvers(context.context) };
        let obj = OwnedJsValue::new(context.context, obj);

        ensure_no_excpetion(context.context)?;

        let obj = obj.try_into_object()?;

        // use .unwrap() here because the fields are guaranteed to be there
        let promise = obj.property("promise")?.unwrap().try_into_promise()?;
        let resolve = obj.property("resolve")?.unwrap().try_into_function()?;
        let reject = obj.property("reject")?.unwrap().try_into_function()?;

        Ok((promise, resolve, reject))
    }
}

impl Deref for OwnedJsPromise {
    type Target = OwnedJsValue;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PromiseState {
    Pending,
    Fulfilled,
    Rejected,
}
