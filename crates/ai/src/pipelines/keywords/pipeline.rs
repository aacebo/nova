use super::config::Config;
use super::{candidates, scorer};
use crate::pipelines::sentence_embeddings::{self, SentenceEmbeddings};
use crate::resources::Result;
use crate::types::Keyword;

/// KeyBERT: embed the document, embed each candidate word, rank candidates by cosine
/// similarity to the document. Any sentence embedder will do.
pub struct Keywords {
    embeddings: SentenceEmbeddings,
    top_n: usize,
}

impl Keywords {
    pub(super) fn new(config: Config) -> Result<Self> {
        let embeddings = sentence_embeddings::Config::default()
            .model(config.model)
            .device(config.device)
            .dtype(config.dtype)
            .build()?;

        Ok(Self {
            embeddings,
            top_n: config.top_n,
        })
    }

    pub fn predict<S: AsRef<str>>(&self, text: &[S]) -> Result<Vec<Vec<Keyword>>> {
        text.iter().map(|text| self.predict_one(text.as_ref())).collect()
    }

    fn predict_one(&self, text: &str) -> Result<Vec<Keyword>> {
        let candidates = candidates::extract(text);

        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        let mut batch: Vec<&str> = Vec::with_capacity(candidates.len() + 1);
        batch.push(text);
        batch.extend(candidates.iter().map(|candidate| candidate.text.as_str()));

        let vectors = self.embeddings.encode(&batch)?;
        let Some((document, vectors)) = vectors.split_first() else {
            return Ok(Vec::new());
        };

        Ok(scorer::rank(candidates, document, vectors, self.top_n))
    }
}
