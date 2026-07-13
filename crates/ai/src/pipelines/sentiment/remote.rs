use crate::clients::openai::OpenAI;
use crate::pipelines::Classify;
use crate::resources::Result;
use crate::types::{Polarity, Sentiment};

const PROMPT: &str = "Classify the sentiment of the text the user gives you. Respond with JSON \
                      matching the schema: `polarity` is either \"positive\" or \"negative\", and \
                      `score` is your confidence from 0.0 to 1.0.";

#[derive(serde::Deserialize)]
struct Output {
    polarity: Polarity,
    score: f64,
}

pub struct Remote {
    client: OpenAI,
}

impl Remote {
    pub fn new(client: OpenAI) -> Self {
        Self { client }
    }
}

impl Classify for Remote {
    fn classify(&self, text: &[&str]) -> Result<Vec<Sentiment>> {
        text.iter()
            .map(|text| {
                let out: Output = self.client.json(PROMPT, text, schema())?;

                Ok(Sentiment {
                    polarity: out.polarity,
                    score: out.score,
                })
            })
            .collect()
    }
}

fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "polarity": { "type": "string", "enum": ["positive", "negative"] },
            "score": { "type": "number" },
        },
        "required": ["polarity", "score"],
        "additionalProperties": false,
    })
}
