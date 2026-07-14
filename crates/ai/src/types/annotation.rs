use nova_core::{Dynamic, Reflect, ToType, ToValue, Type, Value};

use super::Offset;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Annotation {
    pub name: String,
    pub label: String,
    pub text: String,
    pub score: f64,
    pub spans: Vec<Offset>,
}

impl ToType for Annotation {
    fn to_type(&self) -> Type {
        Type::Any
    }
}

impl Reflect for Annotation {
    fn field(&self, name: &str) -> Value<'_> {
        match name {
            "name" => Value::from(self.name.clone()),
            "label" => Value::from(self.label.clone()),
            "text" => Value::from(self.text.clone()),
            "score" => Value::from(self.score),
            "spans" => self.spans.to_value(),
            _ => Value::Undefined,
        }
    }
}

impl ToValue for Annotation {
    fn to_value(&self) -> Value<'_> {
        Value::Dynamic(Dynamic::from_object(self))
    }
}

impl std::fmt::Display for Annotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}): {}", self.name, self.label, self.text)
    }
}
