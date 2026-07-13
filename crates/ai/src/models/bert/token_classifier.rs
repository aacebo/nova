use candle_core::Tensor;
use candle_nn::{Linear, Module, VarBuilder, ops};

use super::config::Config;
use super::model::Bert;
use crate::resources::{Error, Result};

pub struct TokenClassifier {
    bert: Bert,
    classifier: Linear,
    labels: Vec<String>,
}

impl TokenClassifier {
    pub fn new(vars: VarBuilder, config: &Config) -> Result<Self> {
        let labels = config.labels()?;
        let classifier = candle_nn::linear(config.hidden_size, labels.len(), vars.pp("classifier")).map_err(Error::load)?;

        Ok(Self {
            bert: Bert::new(vars, config)?,
            classifier,
            labels,
        })
    }

    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    /// Per-token label probabilities, shaped `(batch, seq, labels)`.
    pub fn forward(&self, ids: &Tensor, mask: &Tensor) -> Result<Tensor> {
        let hidden = self.bert.forward(ids, mask)?;

        self.classifier
            .forward(&hidden)
            .and_then(|logits| ops::softmax(&logits, 2))
            .map_err(Error::inference)
    }
}
