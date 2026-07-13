use crate::pipelines::{sentence_embeddings, summarization};
use crate::routines::Input;
use crate::{Artifact, ArtifactContent};

pub fn summarization(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let input = Input::from_args(args)?;
    let mut artifacts: Vec<Artifact> = Vec::new();
    let text = input
        .text
        .iter()
        .filter(|text| text.split_whitespace().count() >= 8)
        .cloned()
        .collect::<Vec<_>>();

    if text.is_empty() {
        return Ok(nova_core::Value::from(
            artifacts.into_iter().map(nova_core::Value::from_object).collect::<Vec<_>>(),
        ));
    }

    let out = summarization::get()?.summarize(&text)?;

    for summary in out {
        let vector = sentence_embeddings::get()?.encode(&[&summary])?.into_iter().next();

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
