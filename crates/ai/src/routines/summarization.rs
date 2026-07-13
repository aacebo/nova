use crate::routines::{Input, models};
use crate::{Artifact, ArtifactContent};

pub fn summarization(args: &nova::Args, _scope: &nova::Scope) -> Result<nova::Value, Box<dyn std::error::Error>> {
    let input = Input::from_args(args)?;
    let mut artifacts: Vec<Artifact> = Vec::new();
    let text = input
        .text
        .iter()
        .filter(|text| text.split_whitespace().count() >= 8)
        .cloned()
        .collect::<Vec<_>>();

    if text.is_empty() {
        return Ok(nova::Value::from_serialize(&artifacts));
    }

    let out = models::with_summarization(|model| model.summarize(&text))??;

    for summary in out {
        let vector = models::with_embeddings(|model| model.encode(&[&summary]))??
            .into_iter()
            .next();

        artifacts.push(Artifact {
            name: "summary".to_string(),
            value: ArtifactContent::text(summary),
            vector,
        });
    }

    Ok(nova::Value::from_serialize(&artifacts))
}
