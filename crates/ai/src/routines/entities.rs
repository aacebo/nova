use crate::routines::{Input, models};
use crate::{Annotation, Span};

pub fn entity_extraction(
    args: &nova_core::Args,
    _scope: &nova_core::Scope,
) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let input = Input::from_args(args)?;
    let out = models::with_ner(|model| model.predict_full_entities(&input.text))?;
    let mut annotations: Vec<Annotation> = Vec::new();

    for entities in out {
        for entity in entities.into_iter().filter(|e| e.score as f32 >= input.min_score) {
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
                spans: vec![Span::new(entity.offset.begin, entity.offset.end)],
            });
        }
    }

    Ok(nova_core::Value::from_serialize(&annotations))
}
