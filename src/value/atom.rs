use libquickjs_ng_sys as q;

pub struct OwnedJsAtom {
    context: *mut q::JSContext,
    value: q::JSAtom,
}

impl OwnedJsAtom {
    #[inline]
    pub fn new(context: *mut q::JSContext, value: q::JSAtom) -> Self {
        Self { context, value }
    }
}

impl Drop for OwnedJsAtom {
    fn drop(&mut self) {
        unsafe {
            q::JS_FreeAtom(self.context, self.value);
        }
    }
}

impl Clone for OwnedJsAtom {
    fn clone(&self) -> Self {
        unsafe { q::JS_DupAtom(self.context, self.value) };
        Self {
            context: self.context,
            value: self.value,
        }
    }
}
