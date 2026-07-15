use std::sync::Arc;

use nova_reflect::Value;

use crate::{Args, Binding, Diagnostic, Error, Severity, event};

pub trait Context: Send + Sync + std::fmt::Debug + 'static {
    fn trace_id(&self) -> ulid::Ulid;
    fn name(&self) -> &str;
    fn args(&self) -> &Args;
    fn dispatch(&self, source: event::Source);
    fn next(&self, name: &str, args: Args) -> Arc<dyn Context>;

    fn has(&self, key: &str) -> bool;
    fn get(&self, key: &str) -> Option<Binding>;
    fn declare(&self, key: &str, value: Value);
    fn set(&self, key: &str, value: Value) -> Result<(), Error>;
    fn del(&self, key: &str);

    fn render(&self, src: &str) -> Result<String, Error>;
    fn eval(&self, src: &str) -> Result<Value, Error>;

    fn call(&self, name: &str, args: Args) -> Result<Binding, Error> {
        let binding = self
            .get(name)
            .ok_or_else(|| Error::action(self.trace_id(), name, "not found"))?;

        let call = binding
            .as_call()
            .ok_or_else(|| Error::action(self.trace_id(), name, "not callable"))?;

        self.dispatch(event::object::call(name, args.clone()).into());
        call.call(self.next(name, args).as_ref())
    }

    fn emit(&self, diagnostic: Diagnostic) {
        self.dispatch(diagnostic.into());
    }

    fn error(&self, message: &str) {
        self.emit(Diagnostic::new(self.trace_id()).sev(Severity::Error).message(message));
    }
}
