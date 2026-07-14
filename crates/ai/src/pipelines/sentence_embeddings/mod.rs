mod config;
mod local;
mod model;
mod remote;

use std::sync::{Arc, LazyLock};

pub use config::Config;
pub use model::SentenceEmbeddingsModelType;
use nova_core::FromArgs;

use crate::models::ModelRef;
use crate::pipelines::{Cache, Embed, Key, TextArgs, borrow};
use crate::resources::Result;
use crate::{Artifact, ArtifactContent};

static PIPELINES: LazyLock<Cache<dyn Embed>> = LazyLock::new(Cache::new);

pub fn get(model: &ModelRef, api_key: &Option<String>) -> Result<Arc<dyn Embed>> {
    PIPELINES.get_or_build(Key::new(model, api_key), || {
        Config::default().model(model.clone()).api_key(api_key.clone()).build()
    })
}

pub fn embeddings(
    args: &nova_core::Args,
    _scope: &nova_core::Scope,
) -> std::result::Result<nova_core::Value, Box<dyn std::error::Error>> {
    let TextArgs { text, model, api_key } = TextArgs::from_args(args)?;
    let model = model.resolve(SentenceEmbeddingsModelType::AllMiniLmL12V2.model())?;
    let out = get(&model, &api_key)?.embed(&borrow(&text))?;
    let artifacts: Vec<Artifact> = out
        .into_iter()
        .zip(text)
        .map(|(vector, text)| Artifact {
            name: "embedding".to_string(),
            value: ArtifactContent::text(text),
            vector: Some(vector),
        })
        .collect();

    Ok(nova_core::Value::from(
        artifacts.into_iter().map(nova_core::Value::from_object).collect::<Vec<_>>(),
    ))
}
