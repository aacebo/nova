use nova::{Reflect, Value};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Artifact {
    pub name: String,
    pub value: ArtifactContent,
    pub vector: Option<Vec<f32>>,
}

impl Reflect for Artifact {
    fn get_value(self: &std::sync::Arc<Self>, key: &Value) -> Option<Value> {
        let key = key.as_str()?;

        if key == "name" {
            Some((&self.name).into())
        } else if key == "value" {
            Some(match &self.value {
                ArtifactContent::Text(v) => v.into(),
                ArtifactContent::File(v) => v.display().to_string().into(),
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ArtifactContent {
    Text(String),
    File(std::path::PathBuf),
}

impl ArtifactContent {
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    pub fn file(path: std::path::PathBuf) -> Self {
        Self::File(path)
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_file(&self) -> Option<&std::path::PathBuf> {
        match self {
            Self::File(v) => Some(v),
            _ => None,
        }
    }
}

impl std::fmt::Display for ArtifactContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(v) => write!(f, "{v}"),
            Self::File(v) => write!(f, "{}", v.display()),
        }
    }
}
