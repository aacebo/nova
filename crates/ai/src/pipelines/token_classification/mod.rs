mod aggregation;
mod config;
mod local;
mod model;
mod pii;
mod remote;

use std::sync::{Arc, LazyLock};

pub use config::Config;
pub use model::TokenClassificationModelType;
use nova_core::FromArgs;

use crate::models::ModelRef;
use crate::pipelines::{Cache, Extract, Key, ScoredArgs, borrow};
use crate::resources::Result;
use crate::{Annotation, Offset};

static PIPELINES: LazyLock<Cache<dyn Extract>> = LazyLock::new(Cache::new);

pub fn get(model: &ModelRef, api_key: &Option<String>) -> Result<Arc<dyn Extract>> {
    PIPELINES.get_or_build(Key::new(model, api_key), || {
        Config::default().model(model.clone()).api_key(api_key.clone()).build()
    })
}

pub fn entity_extraction(
    args: &nova_core::Args,
    _scope: &nova_core::Scope,
) -> std::result::Result<nova_core::Value, Box<dyn std::error::Error>> {
    let ScoredArgs {
        text,
        min_score,
        model,
        api_key,
    } = ScoredArgs::from_args(args)?;

    let model = model.resolve(TokenClassificationModelType::BertLargeConll03.model())?;
    let out = get(&model, &api_key)?.entities(&borrow(&text))?;
    let mut annotations: Vec<Annotation> = Vec::new();

    for entities in out {
        for entity in entities.into_iter().filter(|e| e.score >= min_score) {
            annotations.push(Annotation {
                name: String::from("entity"),
                label: match entity.label.as_str() {
                    "ORG" => "organization".to_string(),
                    "PER" => "person".to_string(),
                    "LOC" => "location".to_string(),
                    other => other.to_lowercase(),
                },
                text: entity.word,
                score: entity.score,
                spans: vec![Offset::new(entity.offset.begin, entity.offset.end)],
            });
        }
    }

    Ok(nova_core::Value::from(
        annotations.into_iter().map(nova_core::Value::from_object).collect::<Vec<_>>(),
    ))
}

pub fn pii_extraction(
    args: &nova_core::Args,
    _scope: &nova_core::Scope,
) -> std::result::Result<nova_core::Value, Box<dyn std::error::Error>> {
    let ScoredArgs {
        text,
        min_score,
        model,
        api_key,
    } = ScoredArgs::from_args(args)?;

    let model = model.resolve(TokenClassificationModelType::BertLargeConll03.model())?;
    let out = get(&model, &api_key)?.pii(&borrow(&text), min_score)?;
    let mut annotations: Vec<Annotation> = Vec::new();

    for entities in out {
        for entity in entities {
            annotations.push(Annotation {
                name: String::from("pii"),
                label: entity.label,
                text: entity.word,
                score: entity.score,
                spans: vec![Offset::new(entity.offset.begin, entity.offset.end)],
            });
        }
    }

    Ok(nova_core::Value::from(
        annotations.into_iter().map(nova_core::Value::from_object).collect::<Vec<_>>(),
    ))
}
