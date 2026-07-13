use rust_bert::pipelines::sentiment::SentimentPolarity;

use crate::routines::{Input, models};
use crate::{Annotation, Span};

pub fn sentiment(args: &nova::Args, _scope: &nova::Scope) -> Result<nova::Value, Box<dyn std::error::Error>> {
    let input = Input::from_args(args)?;
    let out = models::with_sentiment(|model| model.predict(input.text.iter().map(String::as_str).collect::<Vec<_>>()))?;
    let mut annotations: Vec<Annotation> = Vec::new();

    for (i, sentiment) in out.into_iter().filter(|v| v.score as f32 >= input.min_score).enumerate() {
        let polarity = match sentiment.polarity {
            SentimentPolarity::Negative => "negative",
            SentimentPolarity::Positive => "positive",
        };

        annotations.push(Annotation {
            name: String::from("sentiment"),
            label: polarity.to_string(),
            text: polarity.to_string(),
            score: sentiment.score,
            spans: vec![Span::new(0, input.text[i].len() as u32)],
        });
    }

    Ok(nova::Value::from_serialize(&annotations))
}
