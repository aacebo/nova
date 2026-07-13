use crate::pipelines::sentiment;
use crate::routines::Input;
use crate::types::Polarity;
use crate::{Annotation, Offset};

pub fn sentiment(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let input = Input::from_args(args)?;
    let out = sentiment::get()?.predict(&input.text)?;
    let mut annotations: Vec<Annotation> = Vec::new();

    for (i, sentiment) in out.into_iter().filter(|v| v.score as f32 >= input.min_score).enumerate() {
        let polarity = match sentiment.polarity {
            Polarity::Negative => "negative",
            Polarity::Positive => "positive",
        };

        annotations.push(Annotation {
            name: String::from("sentiment"),
            label: polarity.to_string(),
            text: polarity.to_string(),
            score: sentiment.score,
            spans: vec![Offset::new(0, input.text[i].len() as u32)],
        });
    }

    Ok(nova_core::Value::from(
        annotations.into_iter().map(nova_core::Value::from_object).collect::<Vec<_>>(),
    ))
}
