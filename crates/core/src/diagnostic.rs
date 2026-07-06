#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warn,
    Error,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Diagnostic {
    pub id: ulid::Ulid,
    pub trace_id: ulid::Ulid,
    pub severity: Option<Severity>,
    pub message: Option<String>,
    pub children: Vec<Self>,
}

impl Diagnostic {
    pub fn new(trace_id: ulid::Ulid) -> Self {
        Self {
            id: ulid::Ulid::new(),
            trace_id,
            severity: None,
            message: None,
            children: vec![],
        }
    }

    pub fn sev(mut self, severity: Severity) -> Self {
        self.severity = Some(severity);
        self
    }

    pub fn message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }

    pub fn child(mut self, value: impl Into<Self>) -> Self {
        self.children.push(value.into());
        self
    }

    pub fn severity(&self) -> Severity {
        let own = self.severity.unwrap_or(Severity::Info);
        self.children
            .iter()
            .map(|child| child.severity())
            .chain([own])
            .max()
            .unwrap_or(own)
    }
}
