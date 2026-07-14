pub mod clients;
pub mod models;
pub mod pipelines;
pub mod resources;
pub mod tasks;

mod types;

use nova_core::{Function, Namespace, Pointer, ToType, ToValue, Type, Value};
pub use types::*;

use crate::pipelines::{embeddings, entities, keywords, pii, sentiment, summarize};

pub trait AI {
    fn ai(self) -> Self;
}

impl AI for nova_core::Builder {
    fn ai(self) -> Self {
        self.var("ai", Pointer::namespace(Ai))
    }
}

#[derive(Debug)]
pub struct Ai;

impl ToType for Ai {
    fn to_type(&self) -> Type {
        Type::Any
    }
}

impl ToValue for Ai {
    fn to_value(&self) -> Value<'_> {
        Value::Undefined
    }
}

impl Namespace for Ai {
    fn member(&self, name: &str) -> Option<Pointer> {
        let func = match name {
            "embeddings" => Function::func("ai.embeddings", embeddings),
            "entities" => Function::func("ai.entities", entities),
            "keywords" => Function::func("ai.keywords", keywords),
            "pii" => Function::func("ai.pii", pii),
            "sentiment" => Function::func("ai.sentiment", sentiment),
            "summarize" => Function::func("ai.summarize", summarize),
            _ => return None,
        };

        Some(Pointer::callable(func))
    }

    fn members(&self) -> Vec<String> {
        ["embeddings", "entities", "keywords", "pii", "sentiment", "summarize"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}
