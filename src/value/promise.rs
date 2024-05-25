use std::ops::Deref;

use libquickjspp_sys as q;

use crate::{Context, ExecutionError, JsFunction, ValueError};

use super::OwnedJsValue;

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

    pub fn has_handler(&self) -> bool {
        todo!()
    }

    /// Returns the result of the promise if the promise's state is in the FULFILLED or REJECTED state,
    /// otherwise returns Undefined.
    pub fn result(&self) -> OwnedJsValue {
        let result = unsafe { q::JS_PromiseResult(self.value.context(), self.value.value) };
        OwnedJsValue::new(self.value.context(), result)
    }

    pub fn then(
        &self,
        on_fulfilled: &OwnedJsValue,
        on_rejected: &OwnedJsValue,
    ) -> Result<OwnedJsPromise, ExecutionError> {
        todo!()
    }

    pub fn catch(&self, on_rejected: &OwnedJsValue) -> Result<OwnedJsPromise, ExecutionError> {
        todo!()
    }

    pub fn finally(&self, on_finally: &OwnedJsValue) -> Result<OwnedJsPromise, ExecutionError> {
        todo!()
    }

    pub fn resolve(
        context: &Context,
        value: &OwnedJsValue,
    ) -> Result<OwnedJsPromise, ExecutionError> {
        todo!()
    }

    pub fn reject(
        context: &Context,
        value: &OwnedJsValue,
    ) -> Result<OwnedJsPromise, ExecutionError> {
        todo!()
    }

    pub fn all(
        context: &Context,
        values: &[&OwnedJsValue],
    ) -> Result<OwnedJsPromise, ExecutionError> {
        todo!()
    }

    pub fn all_settled(
        context: &Context,
        values: &[&OwnedJsValue],
    ) -> Result<OwnedJsPromise, ExecutionError> {
        todo!()
    }

    pub fn race(
        context: &Context,
        values: &[&OwnedJsValue],
    ) -> Result<OwnedJsPromise, ExecutionError> {
        todo!()
    }

    pub fn any(
        context: &Context,
        values: &[&OwnedJsValue],
    ) -> Result<OwnedJsPromise, ExecutionError> {
        todo!()
    }

    pub fn with_resolvers(
        context: &Context,
        resolver: &OwnedJsValue,
    ) -> Result<(OwnedJsPromise, JsFunction, JsFunction), ExecutionError> {
        todo!()
    }
}

impl Deref for OwnedJsPromise {
    type Target = OwnedJsValue;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub enum PromiseState {
    Pending,
    Fulfilled,
    Rejected,
}
