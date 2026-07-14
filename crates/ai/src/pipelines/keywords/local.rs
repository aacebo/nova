use super::config::Config;
use super::{Keywords, candidates, scorer};
use crate::pipelines::{Embed, embeddings};
use crate::resources::Result;
use crate::types::Keyword;

/// KeyBERT: embed the document, embed each candidate word, rank candidates by cosine
/// similarity to the document. Any sentence embedder will do.
pub struct Local {
    embeddings: std::sync::Arc<dyn Embed>,
    top_n: usize,
}

impl Local {
    pub fn new(config: Config) -> Result<Self> {
        // Through the shared cache: `ai.embeddings` and `ai.keywords.extract` on the same model
        // then hold one copy of the weights, not two.
        let embeddings = embeddings::get(&config.model, &config.api_key)?;

        Ok(Self {
            embeddings,
            top_n: config.top_n,
        })
    }

    pub fn predict(&self, text: &[&str]) -> Result<Vec<Vec<Keyword>>> {
        text.iter().map(|text| self.predict_one(text)).collect()
    }

    pub fn predict_one(&self, text: &str) -> Result<Vec<Keyword>> {
        let candidates = candidates::extract(text);

        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        let mut batch: Vec<&str> = Vec::with_capacity(candidates.len() + 1);
        batch.push(text);
        batch.extend(candidates.iter().map(|candidate| candidate.text.as_str()));

        let vectors = self.embeddings.embed(&batch)?;
        let Some((document, vectors)) = vectors.split_first() else {
            return Ok(Vec::new());
        };

        Ok(scorer::rank(candidates, document, vectors, self.top_n))
    }
}

impl Keywords for Local {
    fn keywords(&self, text: &[&str]) -> Result<Vec<Vec<Keyword>>> {
        self.predict(text)
    }
}
