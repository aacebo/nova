use crate::{Context, Diagnostic, Value};

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

impl From<Context<'_>> for Output {
    fn from(mut ctx: Context<'_>) -> Self {
        Self::new(*ctx.trace_id()).diagnostics(ctx.take_diagnostics())
    }
}
