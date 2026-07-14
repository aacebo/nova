pub mod clients;
pub mod models;
pub mod pipelines;
pub mod resources;
pub mod tasks;

mod types;

use nova_core::Function;
use nova_template::{Namespace, Pointer};
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
