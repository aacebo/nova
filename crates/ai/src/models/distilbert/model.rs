use candle_core::Tensor;
use candle_nn::VarBuilder;
use candle_transformers::models::distilbert;

use super::config::Config;
use crate::resources::{Error, Result};

pub struct DistilBert {
    inner: distilbert::DistilBertModel,
}

impl DistilBert {
    pub fn new(vars: VarBuilder, config: &Config) -> Result<Self> {
        Ok(Self {
            inner: distilbert::DistilBertModel::load(vars, &config.to_candle()?).map_err(Error::load)?,
        })
    }

    /// `padding` is truthy where a position must be IGNORED -- the inverse of Bert's keep-mask.
    pub fn forward(&self, ids: &Tensor, padding: &Tensor) -> Result<Tensor> {
        self.inner.forward(ids, padding).map_err(Error::inference)
    }
}
