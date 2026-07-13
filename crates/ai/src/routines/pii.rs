use crate::pipelines::token_classification::{self, TokenClassificationCheckpoint};
use crate::routines::args;
use crate::{Annotation, Offset};

pub fn pii_extraction(
    args_: &nova_core::Args,
    _scope: &nova_core::Scope,
) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let text = args::text(args_)?;
    let min_score = args::min_score(args_)?;
    let model = args::model(args_, TokenClassificationCheckpoint::BertLargeConll03.model())?;
    let api_key = args::api_key(args_)?;
    let out = token_classification::get(&model, &api_key)?.pii(&args::borrow(&text), min_score)?;

    let mut annotations: Vec<Annotation> = Vec::new();

    for entities in out {
        for entity in entities {
            annotations.push(Annotation {
                name: String::from("pii"),
                label: entity.label,
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
