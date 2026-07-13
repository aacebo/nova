mod annotation;
mod artifact;
mod routines;
mod span;

use std::sync::Arc;

pub use annotation::*;
pub use artifact::*;
pub use routines::*;
pub use span::*;

pub trait AI {
    fn ai(self) -> Self;
}

impl AI for nova::Builder {
    fn ai(self) -> Self {
        self.var("ai", nova::Value::from_object(Ai))
    }
}

#[derive(Debug)]
pub struct Ai;

impl nova::Reflect for Ai {
    fn get_value(self: &Arc<Self>, key: &nova::Value) -> Option<nova::Value> {
        match key.as_str()? {
            "embeddings" => Some(nova::Value::from_object(nova::Function::func("ai.embeddings", embeddings))),
            "entities" => Some(nova::Value::from_object(Entities)),
            "keywords" => Some(nova::Value::from_object(Keywords)),
            "pii" => Some(nova::Value::from_object(Pii)),
            "sentiment" => Some(nova::Value::from_object(nova::Function::func("ai.sentiment", sentiment))),
            "summarize" => Some(nova::Value::from_object(nova::Function::func("ai.summarize", summarization))),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Entities;

impl nova::Reflect for Entities {
    fn get_value(self: &Arc<Self>, key: &nova::Value) -> Option<nova::Value> {
        match key.as_str()? {
            "extract" => Some(nova::Value::from_object(nova::Function::func(
                "ai.entities.extract",
                entity_extraction,
            ))),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Keywords;

impl nova::Reflect for Keywords {
    fn get_value(self: &Arc<Self>, key: &nova::Value) -> Option<nova::Value> {
        match key.as_str()? {
            "extract" => Some(nova::Value::from_object(nova::Function::func(
                "ai.keywords.extract",
                keyword_extraction,
            ))),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Pii;

impl nova::Reflect for Pii {
    fn get_value(self: &Arc<Self>, key: &nova::Value) -> Option<nova::Value> {
        match key.as_str()? {
            "extract" => Some(nova::Value::from_object(nova::Function::func(
                "ai.pii.extract",
                pii_extraction,
            ))),
            _ => None,
        }
    }
}
