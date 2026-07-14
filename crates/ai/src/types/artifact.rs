use nova_reflect::{Dynamic, Object, ToType, ToValue, Type, Value};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Artifact {
    pub name: String,
    pub value: ArtifactContent,
    pub vector: Option<Vec<f32>>,
}

impl ToType for Artifact {
    fn to_type(&self) -> Type {
        Type::Any
    }
}

impl Object for Artifact {
    fn field(&self, name: &str) -> Value<'_> {
        match name {
            "name" => Value::from(self.name.clone()),
            "value" => Value::from(self.value.to_string()),
            "vector" => match &self.vector {
                Some(v) => v.to_value(),
                None => Value::Null,
            },
            _ => Value::Undefined,
        }
    }
}

impl ToValue for Artifact {
    fn to_value(&self) -> Value<'_> {
        Value::Dynamic(Dynamic::from_object(self))
    }
}

impl std::fmt::Display for Artifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ArtifactContent {
    Text { text: String },
    File { path: std::path::PathBuf },
}

impl ArtifactContent {
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text { text: value.into() }
    }

    pub fn file(path: std::path::PathBuf) -> Self {
        Self::File { path }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text),
            _ => None,
        }
    }

    pub fn as_file(&self) -> Option<&std::path::PathBuf> {
        match self {
            Self::File { path } => Some(path),
            _ => None,
        }
    }
}

impl std::fmt::Display for ArtifactContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text { text } => write!(f, "{text}"),
            Self::File { path } => write!(f, "{}", path.display()),
        }
    }
}
