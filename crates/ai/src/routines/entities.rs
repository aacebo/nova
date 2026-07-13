use crate::pipelines::token_classification::{self, TokenClassificationCheckpoint};
use crate::routines::args;
use crate::{Annotation, Offset};

pub fn entity_extraction(
    args_: &nova_core::Args,
    _scope: &nova_core::Scope,
) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let text = args::text(args_)?;
    let min_score = args::min_score(args_)?;
    let model = args::model(args_, TokenClassificationCheckpoint::BertLargeConll03.model())?;
    let api_key = args::api_key(args_)?;
    let out = token_classification::get(&model, &api_key)?.entities(&args::borrow(&text))?;

    let mut annotations: Vec<Annotation> = Vec::new();

    for entities in out {
        for entity in entities.into_iter().filter(|e| e.score >= min_score) {
            annotations.push(Annotation {
                name: String::from("entity"),
                label: match entity.label.as_str() {
                    "ORG" => "organization".to_string(),
                    "PER" => "person".to_string(),
                    "LOC" => "location".to_string(),
                    other => other.to_lowercase(),
                },
                text: entity.word,
                score: entity.score,
                spans: vec![Offset::new(entity.offset.begin, entity.offset.end)],
            });
        }
    }

    Ok(nova_core::Value::from(
        annotations.into_iter().map(nova_core::Value::from_object).collect::<Vec<_>>(),
    ))
}
