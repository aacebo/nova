use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Error {
    Action(ActionError),
    #[serde(serialize_with = "serialize::to_string")]
    Other(Arc<dyn std::error::Error + Send + Sync>),
}

impl Error {
    pub fn action(trace_id: impl Into<String>, name: impl Into<String>, source: impl Into<ErrorSource>) -> Self {
        Self::Action(ActionError {
            trace_id: trace_id.into(),
            name: name.into(),
            source: source.into(),
        })
    }

    pub fn other(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Other(Arc::new(error))
    }

    pub fn message(text: impl Into<String>) -> Self {
        Self::Other(Arc::new(ErrorSource::from(text.into())))
    }
}

impl From<ActionError> for Error {
    fn from(value: ActionError) -> Self {
        Self::Action(value)
    }
}

impl From<Error> for minijinja::Error {
    fn from(value: Error) -> Self {
        minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, value.to_string())
    }
}

impl From<minijinja::Error> for Error {
    fn from(value: minijinja::Error) -> Self {
        Self::other(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Action(v) => write!(f, "{v}"),
            Self::Other(v) => write!(f, "{v}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Action(v) => Some(v),
            Self::Other(v) => Some(v.as_ref()),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(untagged)]
pub enum ErrorSource {
    Message(String),
    Error(Box<Error>),
}

impl<T: Into<String>> From<T> for ErrorSource {
    fn from(value: T) -> Self {
        Self::Message(value.into())
    }
}

impl From<Error> for ErrorSource {
    fn from(value: Error) -> Self {
        Self::Error(Box::new(value))
    }
}

impl std::fmt::Display for ErrorSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(v) => write!(f, "{v}"),
            Self::Error(v) => write!(f, "{v}"),
        }
    }
}

impl std::error::Error for ErrorSource {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Error(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ActionError {
    pub trace_id: String,
    pub name: String,
    pub source: ErrorSource,
}

impl std::fmt::Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[error::action::{}] => {}", self.name, self.source)
    }
}

impl std::error::Error for ActionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.source()
    }
}

mod serialize {
    pub fn to_string<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: ToString,
    {
        serializer.serialize_str(&value.to_string())
    }
}
