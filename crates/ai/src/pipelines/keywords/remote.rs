use crate::clients::openai::OpenAI;
use crate::pipelines::{Keywords, anchor};
use crate::resources::Result;
use crate::types::Keyword;

const PROMPT: &str = "Extract the salient keywords from the text the user gives you. Respond with \
                      JSON matching the schema. Each `text` must be copied verbatim from the input. \
                      `score` is the keyword's salience from 0.0 to 1.0. Order by descending score.";

#[derive(serde::Deserialize)]
struct Found {
    keywords: Vec<Item>,
}

#[derive(serde::Deserialize)]
struct Item {
    text: String,
    score: f32,
}

pub struct Remote {
    client: OpenAI,
    top_n: usize,
}

impl Remote {
    pub fn new(client: OpenAI, top_n: usize) -> Self {
        Self { client, top_n }
    }
}

impl Keywords for Remote {
    fn keywords(&self, text: &[&str]) -> Result<Vec<Vec<Keyword>>> {
        text.iter()
            .map(|text| {
                let found: Found = self.client.json(PROMPT, text, schema())?;

                Ok(found
                    .keywords
                    .into_iter()
                    .take(self.top_n)
                    .map(|item| Keyword {
                        // A hosted API gives no offsets, so the spans are re-anchored in the source.
                        offsets: anchor::all(text, &item.text),
                        text: item.text,
                        score: item.score,
                    })
                    .collect())
            })
            .collect()
    }
}

fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "keywords": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string" },
                        "score": { "type": "number" },
                    },
                    "required": ["text", "score"],
                    "additionalProperties": false,
                },
            },
        },
        "required": ["keywords"],
        "additionalProperties": false,
    })
}
