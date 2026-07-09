use nova::{Reflect, Value};

use crate::Span;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Annotation {
    pub name: String,
    pub label: String,
    pub text: String,
    pub score: f64,
    pub spans: Vec<Span>,
}

impl Reflect for Annotation {
    fn get_value(self: &std::sync::Arc<Self>, key: &Value) -> Option<Value> {
        let key = key.as_str()?;

        if key == "name" {
            Some((&self.name).into())
        } else if key == "label" {
            Some((&self.label).into())
        } else if key == "text" {
            Some((&self.text).into())
        } else if key == "score" {
            Some(self.score.into())
        } else {
            None
        }
    }
}
