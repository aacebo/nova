use crate::{Diagnostic, Scope, Value};

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub id: ulid::Ulid,
    pub trace_id: ulid::Ulid,
    pub value: Option<Value>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Output {
    pub fn new(trace_id: ulid::Ulid) -> Self {
        Self {
            id: ulid::Ulid::new(),
            trace_id,
            value: None,
            diagnostics: vec![],
        }
    }

    pub fn value(mut self, value: impl Into<Value>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn diagnostics(mut self, diagnostics: Vec<Diagnostic>) -> Self {
        self.diagnostics = diagnostics;
        self
    }
}

impl From<Scope> for Output {
    fn from(scope: Scope) -> Self {
        Self::new(*scope.trace_id()).diagnostics(scope.take_diagnostics())
    }
}
