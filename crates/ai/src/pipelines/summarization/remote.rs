use super::summarize::Summarize;
use crate::clients::openai::OpenAI;
use crate::resources::Result;

const PROMPT: &str = "Summarize the text the user gives you. Respond with JSON matching the schema: \
                      a single field `summary` holding the summary as a string. Be concise and \
                      factual; use only information present in the text.";

#[derive(serde::Deserialize)]
struct Summary {
    summary: String,
}

pub struct Remote {
    client: OpenAI,
}

impl Remote {
    pub fn new(client: OpenAI) -> Self {
        Self { client }
    }
}

impl Summarize for Remote {
    fn summarize(&self, text: &[&str]) -> Result<Vec<String>> {
        text.iter()
            .map(|text| {
                let out: Summary = self.client.json(PROMPT, text, schema())?;
                Ok(out.summary)
            })
            .collect()
    }
}

fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": { "summary": { "type": "string" } },
        "required": ["summary"],
        "additionalProperties": false,
    })
}
