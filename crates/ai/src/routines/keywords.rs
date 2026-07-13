use crate::pipelines::keywords;
use crate::pipelines::sentence_embeddings::SentenceEmbeddingsCheckpoint;
use crate::routines::args;
use crate::{Annotation, Offset};

pub fn keyword_extraction(
    args_: &nova_core::Args,
    _scope: &nova_core::Scope,
) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let text = args::text(args_)?;
    let min_score = args::min_score(args_)? as f32;
    let model = args::model(args_, SentenceEmbeddingsCheckpoint::AllMiniLmL6V2.model())?;
    let api_key = args::api_key(args_)?;
    let out = keywords::get(&model, &api_key)?.keywords(&args::borrow(&text))?;

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
