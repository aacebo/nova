mod config;
mod local;
mod model;
mod remote;

use std::sync::{Arc, LazyLock};

pub use config::Config;
pub use model::SentimentModelType;
use nova_core::FromArgs;

use crate::models::ModelRef;
use crate::pipelines::{Cache, Classify, Key, ScoredArgs, borrow};
use crate::resources::Result;
use crate::{Annotation, Offset};

static PIPELINES: LazyLock<Cache<dyn Classify>> = LazyLock::new(Cache::new);

pub fn get(model: &ModelRef, api_key: &Option<String>) -> Result<Arc<dyn Classify>> {
    PIPELINES.get_or_build(Key::new(model, api_key), || {
        Config::default().model(model.clone()).api_key(api_key.clone()).build()
    })
}

pub fn sentiment(
    args: &nova_core::Args,
    _scope: &nova_core::Scope,
) -> std::result::Result<nova_core::Value, Box<dyn std::error::Error>> {
    let ScoredArgs {
        text,
        min_score,
        model,
        api_key,
    } = ScoredArgs::from_args(args)?;

    let model = model.resolve(SentimentModelType::DistilBertSst2.model())?;
    let out = get(&model, &api_key)?.classify(&borrow(&text))?;
    let mut annotations: Vec<Annotation> = Vec::new();

    for (index, sentiment) in out.into_iter().enumerate() {
        if sentiment.score < min_score {
            continue;
        }

        let source = text.get(index).map(String::as_str).unwrap_or_default();
        let label = sentiment.polarity.as_str();

        annotations.push(Annotation {
            name: String::from("sentiment"),
            label: label.to_string(),
            text: label.to_string(),
            score: sentiment.score,
            spans: vec![Offset::new(0, source.chars().count() as u32)],
        });
    }

    Ok(nova_core::Value::from(
        annotations.into_iter().map(nova_core::Value::from_object).collect::<Vec<_>>(),
    ))
}
