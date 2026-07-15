use nova_reflect::{DynamicRef, Object, ToType, ToValue, Type, ValueRef};

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

impl Object for Annotation {
    fn field_by_ref(&self, name: &str) -> ValueRef<'_> {
        match name {
            "name" => ValueRef::Str(&self.name),
            "label" => ValueRef::Str(&self.label),
            "text" => ValueRef::Str(&self.text),
            "score" => ValueRef::from(self.score),
            "spans" => self.spans.to_value_ref(),
            _ => ValueRef::Undefined,
        }
    }

    fn field(&self, name: &str) -> nova_reflect::Value {
        match name {
            "spans" => {
                let spans: Vec<nova_reflect::Value> = self
                    .spans
                    .iter()
                    .map(|s| nova_reflect::Value::Dynamic(nova_reflect::Dynamic::from_object(std::sync::Arc::new(*s))))
                    .collect();

                nova_reflect::Value::Dynamic(nova_reflect::Dynamic::from_sequence(std::sync::Arc::new(spans)))
            }
            _ => self.field_by_ref(name).to_owned(),
        }
    }
}

impl ToValue for Annotation {
    fn to_value_ref(&self) -> ValueRef<'_> {
        ValueRef::Dynamic(DynamicRef::from_object(self))
    }

    fn to_value(&self) -> nova_reflect::Value {
        nova_reflect::Value::Dynamic(nova_reflect::Dynamic::from_object(std::sync::Arc::new(self.clone())))
    }
}

impl std::fmt::Display for Annotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}): {}", self.name, self.label, self.text)
    }
}
