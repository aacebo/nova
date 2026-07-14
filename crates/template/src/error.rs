use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Error {
    message: String,
    line: Option<usize>,
    source: Option<Arc<dyn std::error::Error + Send + Sync>>,
}

impl Error {
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: None,
            source: None,
        }
    }

    pub fn wrap(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self {
            message: error.to_string(),
            line: None,
            source: Some(Arc::new(error)),
        }
    }

    pub fn at_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    pub fn line(&self) -> Option<usize> {
        self.line
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.line {
            Some(line) => write!(f, "[template:{line}] {}", self.message),
            None => write!(f, "[template] {}", self.message),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}
