use std::collections::BTreeMap;

pub fn group() -> ErrorGroup {
    ErrorGroup::default()
}

pub fn object() -> ObjectError {
    ObjectError::default()
}

pub fn list() -> ListError {
    ListError::default()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Error {
    List(ListError),
    Object(ObjectError),
    Group(ErrorGroup),
    Custom { error: Box<Self>, message: String },
    Rule { name: String, message: String },
}

impl<K: std::fmt::Display, M: std::fmt::Display> From<(K, M)> for Error {
    fn from(value: (K, M)) -> Self {
        Self::Rule {
            name: value.0.to_string(),
            message: value.1.to_string(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rule { name, message } => write!(f, "{}: {}", name, message),
            Self::List(err) => write!(f, "{err}"),
            Self::Group(err) => write!(f, "{err}"),
            Self::Object(err) => write!(f, "{err}"),
            Self::Custom { error: _, message } => write!(f, "{message}"),
        }
    }
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ErrorGroup(Vec<Error>);

impl ErrorGroup {
    pub fn ok(self) -> Result<(), Error> {
        if self.0.is_empty() { Ok(()) } else { Err(self.into()) }
    }
}

impl From<ErrorGroup> for Error {
    fn from(value: ErrorGroup) -> Self {
        Self::Group(value)
    }
}

impl std::ops::Deref for ErrorGroup {
    type Target = Vec<Error>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ErrorGroup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for ErrorGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for err in &self.0 {
            write!(f, "{err}")?;
        }

        Ok(())
    }
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ObjectError(BTreeMap<String, Vec<Error>>);

impl ObjectError {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn ok(self) -> Result<(), Error> {
        if self.0.is_empty() { Ok(()) } else { Err(self.into()) }
    }

    pub fn field(mut self, name: impl std::fmt::Display, error: Error) -> Self {
        let errors = self.0.entry(name.to_string()).or_default();
        errors.push(error);
        self
    }
}

impl From<ObjectError> for Error {
    fn from(value: ObjectError) -> Self {
        Self::Object(value)
    }
}

impl std::fmt::Display for ObjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, errs) in &self.0 {
            for err in errs {
                writeln!(f, "{k}: {err}")?;
            }
        }

        Ok(())
    }
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ListError(BTreeMap<usize, Vec<Error>>);

impl ListError {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn ok(self) -> Result<(), Error> {
        if self.0.is_empty() { Ok(()) } else { Err(self.into()) }
    }

    pub fn index(mut self, index: usize, error: Error) -> Self {
        let errors = self.0.entry(index).or_default();
        errors.push(error);
        self
    }
}

impl From<ListError> for Error {
    fn from(value: ListError) -> Self {
        Self::List(value)
    }
}

impl std::fmt::Display for ListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, errs) in &self.0 {
            writeln!(f, "{k}")?;

            for err in errs {
                writeln!(f, "{err}")?;
            }
        }

        Ok(())
    }
}
