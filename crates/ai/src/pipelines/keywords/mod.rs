mod candidates;
mod config;
mod local;
mod remote;
mod scorer;
mod stopwords;

use std::sync::{Arc, LazyLock};

pub use config::Config;
use nova_core::FromArgs;

use crate::models::ModelRef;
use crate::pipelines::sentence_embeddings::SentenceEmbeddingsModelType;
use crate::pipelines::{Cache, Key, Keywords, ScoredArgs, borrow};
use crate::resources::Result;
use crate::{Annotation, Offset};

static PIPELINES: LazyLock<Cache<dyn Keywords>> = LazyLock::new(Cache::new);

pub fn get(model: &ModelRef, api_key: &Option<String>) -> Result<Arc<dyn Keywords>> {
    PIPELINES.get_or_build(Key::new(model, api_key), || {
        Config::default().model(model.clone()).api_key(api_key.clone()).build()
    })
}

pub fn keyword_extraction(
    args: &nova_core::Args,
    _scope: &nova_core::Scope,
) -> std::result::Result<nova_core::Value, Box<dyn std::error::Error>> {
    let ScoredArgs {
        text,
        min_score,
        model,
        api_key,
    } = ScoredArgs::from_args(args)?;

    let min_score = min_score as f32;
    let model = model.resolve(SentenceEmbeddingsModelType::AllMiniLmL6V2.model())?;
    let out = get(&model, &api_key)?.keywords(&borrow(&text))?;
    let mut annotations: Vec<Annotation> = Vec::new();

    for keywords in out {
        for keyword in keywords.into_iter().filter(|k| k.score >= min_score) {
            annotations.push(Annotation {
                name: String::from("keyword"),
                label: keyword.text.clone(),
                text: keyword.text,
                score: keyword.score as f64,
                spans: keyword.offsets.iter().map(|o| Offset::new(o.begin, o.end)).collect(),
            });
        }
    }

    Ok(nova_core::Value::from(
        annotations.into_iter().map(nova_core::Value::from_object).collect::<Vec<_>>(),
    ))
}
