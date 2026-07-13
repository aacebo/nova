use crate::pipelines::sentence_embeddings;
use crate::routines::Input;
use crate::{Artifact, ArtifactContent};

pub fn embeddings(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let input = Input::from_args(args)?;
    let out = sentence_embeddings::get()?.encode(&input.text)?;
    let mut artifacts: Vec<Artifact> = Vec::new();

    for (vector, text) in out.into_iter().zip(input.text) {
        artifacts.push(Artifact {
            name: "embedding".to_string(),
            value: ArtifactContent::text(text),
            vector: Some(vector),
        });
    }

    Ok(nova_core::Value::from(
        artifacts.into_iter().map(nova_core::Value::from_object).collect::<Vec<_>>(),
    ))
}
