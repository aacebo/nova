pub mod clients;
pub mod models;
pub mod pipelines;
pub mod resources;

mod types;

use std::sync::Arc;

pub use types::*;

use crate::pipelines::{embeddings, keywords, sentiment, summarize, token_classification};

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
        match key.as_str()? {
            "embeddings" => Some(nova_core::Value::from_object(nova_core::Function::func(
                "ai.embeddings",
                embeddings::run,
            ))),
            "entities" => Some(nova_core::Value::from_object(Entities)),
            "keywords" => Some(nova_core::Value::from_object(Keywords)),
            "pii" => Some(nova_core::Value::from_object(Pii)),
            "sentiment" => Some(nova_core::Value::from_object(nova_core::Function::func(
                "ai.sentiment",
                sentiment::run,
            ))),
            "summarize" => Some(nova_core::Value::from_object(nova_core::Function::func(
                "ai.summarize",
                summarize::run,
            ))),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Entities;

impl nova_core::Reflect for Entities {
    fn get_value(self: &Arc<Self>, key: &nova_core::Value) -> Option<nova_core::Value> {
        match key.as_str()? {
            "extract" => Some(nova_core::Value::from_object(nova_core::Function::func(
                "ai.entities.extract",
                token_classification::extract_entities,
            ))),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Keywords;

impl nova_core::Reflect for Keywords {
    fn get_value(self: &Arc<Self>, key: &nova_core::Value) -> Option<nova_core::Value> {
        match key.as_str()? {
            "extract" => Some(nova_core::Value::from_object(nova_core::Function::func(
                "ai.keywords.extract",
                keywords::run,
            ))),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Pii;

impl nova_core::Reflect for Pii {
    fn get_value(self: &Arc<Self>, key: &nova_core::Value) -> Option<nova_core::Value> {
        match key.as_str()? {
            "extract" => Some(nova_core::Value::from_object(nova_core::Function::func(
                "ai.pii.extract",
                token_classification::extract_pii,
            ))),
            _ => None,
        }
    }
}
