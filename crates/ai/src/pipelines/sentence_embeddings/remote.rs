use super::embed::Embed;
use crate::clients::openai::OpenAI;
use crate::resources::Result;

pub struct Remote {
    client: OpenAI,
}

impl Remote {
    pub fn new(client: OpenAI) -> Self {
        Self { client }
    }
}

impl Embed for Remote {
    fn embed(&self, text: &[&str]) -> Result<Vec<Vec<f32>>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        self.client.embeddings(text)
    }
}
