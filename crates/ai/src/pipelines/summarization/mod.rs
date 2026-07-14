mod config;
mod local;
mod model;
mod remote;

use std::sync::{Arc, LazyLock};

pub use config::Config;
pub use model::SummarizationModelType;
use nova_core::FromArgs;

use crate::models::ModelRef;
use crate::pipelines::sentence_embeddings::{self, SentenceEmbeddingsModelType};
use crate::pipelines::{Cache, Key, Summarize, TextArgs, borrow};
use crate::resources::Result;
use crate::{Artifact, ArtifactContent};

const MIN_WORDS: usize = 8;

static PIPELINES: LazyLock<Cache<dyn Summarize>> = LazyLock::new(Cache::new);

pub fn get(model: &ModelRef, api_key: &Option<String>) -> Result<Arc<dyn Summarize>> {
    PIPELINES.get_or_build(Key::new(model, api_key), || {
        Config::default().model(model.clone()).api_key(api_key.clone()).build()
    })
}

pub fn summarization(
    args: &nova_core::Args,
    _scope: &nova_core::Scope,
) -> std::result::Result<nova_core::Value, Box<dyn std::error::Error>> {
    let TextArgs { text, model, api_key } = TextArgs::from_args(args)?;
    let text: Vec<String> = text
        .into_iter()
        .filter(|text| text.split_whitespace().count() >= MIN_WORDS)
        .collect();

    if text.is_empty() {
        return Ok(nova_core::Value::from(Vec::<nova_core::Value>::new()));
    }

    let model = model.resolve(SummarizationModelType::BartLargeCnn.model())?;
    let out = get(&model, &api_key)?.summarize(&borrow(&text))?;
    let embedder = SentenceEmbeddingsModelType::AllMiniLmL12V2.model();
    let embeddings = sentence_embeddings::get(&embedder, &None)?;
    let mut artifacts: Vec<Artifact> = Vec::new();

    for summary in out {
        let vector = embeddings.embed(&[summary.as_str()])?.into_iter().next();

        artifacts.push(Artifact {
            name: "summary".to_string(),
            value: ArtifactContent::text(summary),
            vector,
        });
    }

    Ok(nova_core::Value::from(
        artifacts.into_iter().map(nova_core::Value::from_object).collect::<Vec<_>>(),
    ))
}
