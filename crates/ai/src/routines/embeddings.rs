use crate::pipelines::sentence_embeddings::{self, SentenceEmbeddingsCheckpoint};
use crate::routines::args;
use crate::{Artifact, ArtifactContent};

pub fn embeddings(args_: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let text = args::text(args_)?;
    let model = args::model(args_, SentenceEmbeddingsCheckpoint::AllMiniLmL12V2.model())?;
    let api_key = args::api_key(args_)?;
    let out = sentence_embeddings::get(&model, &api_key)?.embed(&args::borrow(&text))?;

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
