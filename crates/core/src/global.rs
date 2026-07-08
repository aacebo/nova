use std::cell::RefCell;

use crate::{Diagnostic, Scope, Severity};

thread_local! {
    static SCOPE: RefCell<Option<Scope>> = const { RefCell::new(None) };
}

pub fn scope() -> Scope {
    SCOPE
        .with(|scope| scope.borrow().clone())
        .expect("scope macro used outside an invocation")
}

pub fn emit(messages: impl IntoIterator<Item = impl Into<String>>) {
    for message in messages {
        Diagnostic::new(trace_id()).sev(Severity::Error).message(message).emit();
    }
}

pub fn trace_id() -> ulid::Ulid {
    match SCOPE.with(|scope| scope.borrow().as_ref().map(|s| *s.trace_id())) {
        Some(id) => id,
        None => ulid::Ulid::new(),
    }
}

pub fn enter(scope: &Scope) -> Guard {
    let previous = SCOPE.with(|current| current.replace(Some(scope.clone())));
    Guard { previous }
}

pub struct Guard {
    previous: Option<Scope>,
}

impl Drop for Guard {
    fn drop(&mut self) {
        SCOPE.with(|current| *current.borrow_mut() = self.previous.take());
    }
}
