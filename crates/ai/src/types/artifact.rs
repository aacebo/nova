use nova_reflect::{DynamicRef, Object, ToType, ToValue, Type, ValueRef};

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
    fn field_by_ref(&self, name: &str) -> ValueRef<'_> {
        match name {
            "name" => ValueRef::Str(&self.name),
            "value" => match self.value.as_str() {
                Some(text) => ValueRef::Str(text),
                None => ValueRef::Undefined,
            },
            "vector" => match &self.vector {
                Some(v) => v.to_value_ref(),
                None => ValueRef::Null,
            },
            _ => ValueRef::Undefined,
        }
    }
}

impl ToValue for Artifact {
    fn to_value_ref(&self) -> ValueRef<'_> {
        ValueRef::Dynamic(DynamicRef::from_object(self))
    }

    fn to_value(&self) -> nova_reflect::Value {
        nova_reflect::Value::Dynamic(nova_reflect::Dynamic::from_object(std::sync::Arc::new(self.clone())))
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

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text),
            Self::File { path } => path.to_str(),
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
