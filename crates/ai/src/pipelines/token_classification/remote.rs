use crate::clients::openai::OpenAI;
use crate::pipelines::{Extract, anchor};
use crate::resources::Result;
use crate::types::Entity;

const ENTITIES: &str = "Extract the named entities from the text the user gives you. Respond with \
                        JSON matching the schema. `label` is one of person, organization, location, \
                        misc. `text` must be copied verbatim from the input. `score` is your \
                        confidence from 0.0 to 1.0.";

const PII: &str = "Extract personally identifying information from the text the user gives you. \
                   Respond with JSON matching the schema. `label` names the kind of identifier \
                   (person, email, phone, address, ssn, credit_card, ...). `text` must be copied \
                   verbatim from the input. `score` is your confidence from 0.0 to 1.0.";

#[derive(serde::Deserialize)]
struct Found {
    entities: Vec<Item>,
}

#[derive(serde::Deserialize)]
struct Item {
    label: String,
    text: String,
    score: f64,
}

pub struct Remote {
    client: OpenAI,
}

impl Remote {
    pub fn new(client: OpenAI) -> Self {
        Self { client }
    }

    fn run(&self, prompt: &str, text: &[&str]) -> Result<Vec<Vec<Entity>>> {
        text.iter()
            .map(|text| {
                let found: Found = self.client.json(prompt, text, schema())?;

                Ok(found
                    .entities
                    .into_iter()
                    // A hosted API gives no offsets, so spans are re-anchored in the source. An
                    // entity that is not verbatim cannot be anchored -- and violates the prompt --
                    // so it is dropped rather than given an invented span.
                    .filter_map(|item| {
                        Some(Entity {
                            offset: anchor::find(text, &item.text)?,
                            word: item.text,
                            label: item.label.to_lowercase(),
                            score: item.score,
                        })
                    })
                    .collect())
            })
            .collect()
    }
}

impl Extract for Remote {
    fn entities(&self, text: &[&str]) -> Result<Vec<Vec<Entity>>> {
        self.run(ENTITIES, text)
    }

    fn pii(&self, text: &[&str], min_score: f64) -> Result<Vec<Vec<Entity>>> {
        let mut out = self.run(PII, text)?;

        for entities in &mut out {
            entities.retain(|entity| entity.score >= min_score);
        }

        Ok(out)
    }
}

fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "entities": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "label": { "type": "string" },
                        "text": { "type": "string" },
                        "score": { "type": "number" },
                    },
                    "required": ["label", "text", "score"],
                    "additionalProperties": false,
                },
            },
        },
        "required": ["entities"],
        "additionalProperties": false,
    })
}
