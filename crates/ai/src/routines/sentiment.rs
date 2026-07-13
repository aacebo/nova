use crate::pipelines::sentiment::{self, SentimentCheckpoint};
use crate::routines::args;
use crate::{Annotation, Offset};

pub fn sentiment(args_: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let text = args::text(args_)?;
    let min_score = args::min_score(args_)?;
    let model = args::model(args_, SentimentCheckpoint::DistilBertSst2.model())?;
    let api_key = args::api_key(args_)?;
    let out = sentiment::get(&model, &api_key)?.classify(&args::borrow(&text))?;

    let mut annotations: Vec<Annotation> = Vec::new();

    for (index, sentiment) in out.into_iter().enumerate() {
        if sentiment.score < min_score {
            continue;
        }

        let source = text.get(index).map(String::as_str).unwrap_or_default();
        let label = sentiment.polarity.as_str();

        annotations.push(Annotation {
            name: String::from("sentiment"),
            label: label.to_string(),
            text: label.to_string(),
            score: sentiment.score,
            spans: vec![Offset::new(0, source.chars().count() as u32)],
        });
    }

    Ok(nova_core::Value::from(
        annotations.into_iter().map(nova_core::Value::from_object).collect::<Vec<_>>(),
    ))
}
