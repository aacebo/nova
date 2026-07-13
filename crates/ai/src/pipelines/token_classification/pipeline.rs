use candle_core::{Device, Tensor};
use tokenizers::Tokenizer;

use super::aggregation::{self, Word};
use super::config::Config;
use crate::models::bert;
use crate::resources::{Error, Repo, Result};
use crate::types::{Entity, Token};

pub struct TokenClassification {
    classifier: bert::TokenClassifier,
    tokenizer: Tokenizer,
    device: Device,
}

impl TokenClassification {
    pub(super) fn new(config: Config) -> Result<Self> {
        let repo = Repo::open(config.model, config.device, config.dtype)?;
        let model: bert::Config = repo.config()?;
        let device = repo.device().clone();

        Ok(Self {
            classifier: bert::TokenClassifier::new(repo.vars()?, &model)?,
            tokenizer: repo.tokenizer()?,
            device,
        })
    }

    pub fn predict<S: AsRef<str>>(&self, text: &[S]) -> Result<Vec<Vec<Token>>> {
        text.iter()
            .map(|text| {
                let text = text.as_ref();
                Ok(aggregation::tokens(self.words(text)?, text))
            })
            .collect()
    }

    pub fn predict_entities<S: AsRef<str>>(&self, text: &[S]) -> Result<Vec<Vec<Entity>>> {
        text.iter()
            .map(|text| {
                let text = text.as_ref();
                Ok(aggregation::entities(self.words(text)?, text))
            })
            .collect()
    }

    fn words(&self, text: &str) -> Result<Vec<Word>> {
        let encoding = self.tokenizer.encode(text, true).map_err(Error::tokenize)?;
        let ids = encoding.get_ids();

        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let shape = (1, ids.len());
        let input = Tensor::from_slice(ids, shape, &self.device).map_err(Error::inference)?;
        let mask = Tensor::from_slice(encoding.get_attention_mask(), shape, &self.device).map_err(Error::inference)?;

        let probs = self
            .classifier
            .forward(&input, &mask)?
            .squeeze(0)
            .and_then(|probs| probs.to_vec2::<f32>())
            .map_err(Error::inference)?;

        aggregation::words(&probs, &encoding, self.classifier.labels())
    }
}
