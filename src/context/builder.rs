use super::Context;
use crate::{console, ContextError};

/// A builder for [Context](Context).
///
/// Create with [Context::builder](Context::builder).
#[derive(Default)]
pub struct ContextBuilder {
    memory_limit: Option<usize>,
    console_backend: Option<Box<dyn console::ConsoleBackend>>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            memory_limit: None,
            console_backend: None,
        }
    }

    /// Sets the memory limit of the Javascript runtime (in bytes).
    ///
    /// If the limit is exceeded, methods like `eval` will return
    /// a `Err(ExecutionError::Exception(JsValue::Null))`
    // TODO: investigate why we don't get a proper exception message here.
    pub fn memory_limit(self, max_bytes: usize) -> Self {
        let mut s = self;
        s.memory_limit = Some(max_bytes);
        s
    }

    /// Set a console handler that will proxy `console.{log,trace,debug,...}`
    /// calls.
    ///
    /// The given argument must implement the [console::ConsoleBackend] trait.
    ///
    /// A very simple logger could look like this:
    pub fn console<B>(mut self, backend: B) -> Self
    where
        B: console::ConsoleBackend,
    {
        self.console_backend = Some(Box::new(backend));
        self
    }

    /// Finalize the builder and build a JS Context.
    pub fn build(self) -> Result<Context, ContextError> {
        let context = Context::new(self.memory_limit)?;
        if let Some(be) = self.console_backend {
            context.set_console(be).map_err(ContextError::Execution)?;
        }
        Ok(context)
    }
}
