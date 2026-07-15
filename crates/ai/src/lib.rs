pub mod clients;
pub mod models;
pub mod pipelines;
pub mod resources;
pub mod tasks;

mod types;

use nova_core::{Binding, Function, Namespace};
pub use types::*;

use crate::pipelines::{embeddings, entities, keywords, pii, sentiment, summarize};

#[derive(Debug)]
pub struct Ai;

impl Namespace for Ai {
    fn member(&self, name: &str) -> Option<Binding> {
        let func = match name {
            "embeddings" => Function::func("ai.embeddings", embeddings),
            "entities" => Function::func("ai.entities", entities),
            "keywords" => Function::func("ai.keywords", keywords),
            "pii" => Function::func("ai.pii", pii),
            "sentiment" => Function::func("ai.sentiment", sentiment),
            "summarize" => Function::func("ai.summarize", summarize),
            _ => return None,
        };

        Some(Binding::callable(func))
    }

    fn members(&self) -> Vec<String> {
        ["embeddings", "entities", "keywords", "pii", "sentiment", "summarize"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}
