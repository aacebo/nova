pub mod clients;
pub mod models;
pub mod pipelines;
pub mod resources;
pub mod tasks;

mod types;

use std::sync::Arc;

pub use types::*;

use crate::pipelines::{embeddings, entities, keywords, pii, sentiment, summarize};

pub trait AI {
    fn ai(self) -> Self;
}

impl AI for nova_core::Builder {
    fn ai(self) -> Self {
        self.var("ai", nova_core::Value::from_object(Ai))
    }
}

#[derive(Debug)]
pub struct Ai;

impl nova_core::Reflect for Ai {
    fn get_value(self: &Arc<Self>, key: &nova_core::Value) -> Option<nova_core::Value> {
        let func = match key.as_str()? {
            "embeddings" => nova_core::Function::func("ai.embeddings", embeddings),
            "entities" => nova_core::Function::func("ai.entities", entities),
            "keywords" => nova_core::Function::func("ai.keywords", keywords),
            "pii" => nova_core::Function::func("ai.pii", pii),
            "sentiment" => nova_core::Function::func("ai.sentiment", sentiment),
            "summarize" => nova_core::Function::func("ai.summarize", summarize),
            _ => return None,
        };

        Some(nova_core::Value::from_object(func))
    }
}
