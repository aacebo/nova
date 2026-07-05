#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Info,
    Warn,
    Error,
}

impl Level {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Diagnostic {
    pub trace_id: ulid::Ulid,
    pub level: Level,
    pub message: Option<String>,
    pub children: Vec<Self>,
}

impl Diagnostic {
    pub fn new(trace_id: ulid::Ulid, level: Level) -> Self {
        Self {
            trace_id,
            level,
            message: None,
            children: vec![],
        }
    }

    pub fn message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }

    pub fn child(mut self, value: impl Into<Self>) -> Self {
        self.children.push(value.into());
        self
    }
}
