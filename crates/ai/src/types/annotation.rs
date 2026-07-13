use nova_core::{Reflect, Value};

use super::Offset;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Annotation {
    pub name: String,
    pub label: String,
    pub text: String,
    pub score: f64,
    pub spans: Vec<Offset>,
}

impl Reflect for Annotation {
    fn get_value(self: &std::sync::Arc<Self>, key: &Value) -> Option<Value> {
        match key.as_str()? {
            "name" => Some((&self.name).into()),
            "label" => Some((&self.label).into()),
            "text" => Some((&self.text).into()),
            "score" => Some(self.score.into()),
            "spans" => Some(Value::from_serialize(&self.spans)),
            _ => None,
        }
    }
}

impl std::fmt::Display for Annotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}): {}", self.name, self.label, self.text)
    }
}
