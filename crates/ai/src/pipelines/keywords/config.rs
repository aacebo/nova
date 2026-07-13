use candle_core::{DType, Device};

use super::pipeline::Keywords;
use crate::pipelines::sentence_embeddings::SentenceEmbeddingsModel;
use crate::resources::{self, ModelResource, Result};

const TOP_N: usize = 5;

pub struct Config {
    pub model: ModelResource,
    pub device: Device,
    pub dtype: DType,
    pub top_n: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: SentenceEmbeddingsModel::AllMiniLmL6V2.resource(),
            device: resources::default_device(),
            dtype: resources::default_dtype(),
            top_n: TOP_N,
        }
    }
}

impl Config {
    pub fn model(mut self, model: impl Into<ModelResource>) -> Self {
        self.model = model.into();
        self
    }

    pub fn device(mut self, device: Device) -> Self {
        self.device = device;
        self
    }

    pub fn dtype(mut self, dtype: DType) -> Self {
        self.dtype = dtype;
        self
    }

    pub fn top_n(mut self, top_n: usize) -> Self {
        self.top_n = top_n;
        self
    }

    pub fn build(self) -> Result<Keywords> {
        Keywords::new(self)
    }
}
