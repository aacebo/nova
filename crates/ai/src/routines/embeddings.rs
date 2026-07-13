use crate::routines::{Input, models};
use crate::{Artifact, ArtifactContent};

pub fn embeddings(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let input = Input::from_args(args)?;
    let out = models::with_embeddings(|model| model.encode(&input.text))??;
    let mut artifacts: Vec<Artifact> = Vec::new();

    for (vector, text) in out.into_iter().zip(input.text) {
        artifacts.push(Artifact {
            name: "embedding".to_string(),
            value: ArtifactContent::text(text),
            vector: Some(vector),
        });
    }

    Ok(nova_core::Value::from_serialize(&artifacts))
}
