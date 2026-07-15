use std::sync::Arc;

use nova_reflect::Value;

use crate::{Args, Binding, Diagnostic, Error, KArgs, Severity, event};

pub trait Context: Send + Sync + std::fmt::Debug {
    fn trace_id(&self) -> ulid::Ulid;
    fn name(&self) -> &str;
    fn args(&self) -> &[Value];
    fn kargs(&self) -> &KArgs;
    fn resolve(&self, name: &str) -> Option<Binding>;
    fn names(&self) -> Vec<String>;
    fn call(&self, name: &str, args: Args) -> Result<Binding, Error>;
    fn eval(&self, expr: &str) -> Result<Binding, Error>;
    fn render(&self, name: &str) -> Result<String, Error>;
    fn render_str(&self, source: &str) -> Result<String, Error>;
    fn dispatch(&self, source: event::Source);
    fn fork(&self, name: &str, args: Vec<Value>, kargs: KArgs) -> Arc<dyn Context>;
    fn as_any(&self) -> &dyn std::any::Any;

    fn emit(&self, diagnostic: Diagnostic) {
        self.dispatch(diagnostic.into());
    }

    fn error(&self, message: &str) {
        self.emit(Diagnostic::new(self.trace_id()).sev(Severity::Error).message(message));
    }
}

impl dyn Context + '_ {
    pub fn cast<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }

    pub fn is<T: 'static>(&self) -> bool {
        self.as_any().is::<T>()
    }
}
