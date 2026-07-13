use candle_core::{IndexOp, Tensor};
use candle_nn::{Linear, Module, VarBuilder, ops};

use super::config::Config;
use super::model::DistilBert;
use crate::models::Forward;
use crate::resources::{Error, Result};

const LABELS: usize = 2;

pub struct SequenceClassifier {
    distilbert: DistilBert,
    pre_classifier: Linear,
    classifier: Linear,
}

impl SequenceClassifier {
    pub fn new(vars: VarBuilder, config: &Config) -> Result<Self> {
        let hidden = config.hidden_size();

        Ok(Self {
            distilbert: DistilBert::new(vars.clone(), config)?,
            pre_classifier: candle_nn::linear(hidden, hidden, vars.pp("pre_classifier")).map_err(Error::load)?,
            classifier: candle_nn::linear(hidden, LABELS, vars.pp("classifier")).map_err(Error::load)?,
        })
    }

    pub fn forward(&self, ids: &Tensor, padding: &Tensor) -> Result<Vec<Vec<f32>>> {
        let hidden = self.distilbert.forward(ids, padding)?;
        let pooled = hidden.i((.., 0)).map_err(Error::inference)?;

        self.pre_classifier
            .forward(&pooled)
            .and_then(|v| v.relu())
            .and_then(|v| self.classifier.forward(&v))
            .and_then(|logits| ops::softmax(&logits, 1))
            .and_then(|probs| probs.to_vec2::<f32>())
            .map_err(Error::inference)
    }
}

impl Forward for SequenceClassifier {
    type Input = (Tensor, Tensor);
    type Output = Vec<Vec<f32>>;

    fn forward(&self, (ids, padding): Self::Input) -> Result<Self::Output> {
        self.forward(&ids, &padding)
    }
}
