use candle_core::{Device, Tensor};
use tokenizers::Tokenizer;

use super::config::Config;
use super::{Extract, aggregation, pii};
use crate::models::bert;
use crate::resources::{Error, Result};
use crate::types::Entity;

pub struct Local {
    classifier: bert::TokenClassifier,
    tokenizer: Tokenizer,
    device: Device,
}

impl Local {
    pub fn new(config: Config) -> Result<Self> {
        let repo = config.model.loader(config.device.clone(), config.dtype)?;
        let model: bert::Config = repo.config()?;
        let device = repo.device().clone();

        Ok(Self {
            classifier: bert::TokenClassifier::new(repo.vars()?, &model)?,
            tokenizer: repo.tokenizer()?,
            device,
        })
    }

    pub fn ner(&self, text: &[&str]) -> Result<Vec<Vec<Entity>>> {
        text.iter()
            .map(|text| Ok(aggregation::entities(self.words(text)?, text)))
            .collect()
    }

    pub fn identifiers(&self, text: &[&str], min_score: f64) -> Result<Vec<Vec<Entity>>> {
        text.iter()
            .map(|text| Ok(pii::entities(self.words(text)?, text, min_score)))
            .collect()
    }

    pub fn words(&self, text: &str) -> Result<Vec<aggregation::Word>> {
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

impl Extract for Local {
    fn entities(&self, text: &[&str]) -> Result<Vec<Vec<Entity>>> {
        self.ner(text)
    }

    fn pii(&self, text: &[&str], min_score: f64) -> Result<Vec<Vec<Entity>>> {
        self.identifiers(text, min_score)
    }
}
