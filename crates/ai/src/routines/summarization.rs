use crate::pipelines::sentence_embeddings::{self, SentenceEmbeddingsCheckpoint};
use crate::pipelines::summarization::{self, SummarizationCheckpoint};
use crate::routines::args;
use crate::{Artifact, ArtifactContent};

const MIN_WORDS: usize = 8;

pub fn summarization(args_: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let text: Vec<String> = args::text(args_)?
        .into_iter()
        .filter(|text| text.split_whitespace().count() >= MIN_WORDS)
        .collect();

    if text.is_empty() {
        return Ok(nova_core::Value::from(Vec::<nova_core::Value>::new()));
    }

    let model = args::model(args_, SummarizationCheckpoint::BartLargeCnn.model())?;
    let api_key = args::api_key(args_)?;
    let out = summarization::get(&model, &api_key)?.summarize(&args::borrow(&text))?;

    // `provider=` selects the summarizer, not the embedder: routing the summary's vector through
    // a chat model would ask it for embeddings.
    let embedder = SentenceEmbeddingsCheckpoint::AllMiniLmL12V2.model();
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
