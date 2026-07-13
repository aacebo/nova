use candle_core::{DType, Device};

use super::model::SentenceEmbeddingsModel;
use super::pipeline::SentenceEmbeddings;
use crate::resources::{self, ModelResource, Result};

pub struct Config {
    pub model: ModelResource,
    pub device: Device,
    pub dtype: DType,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: SentenceEmbeddingsModel::AllMiniLmL12V2.resource(),
            device: resources::default_device(),
            dtype: resources::default_dtype(),
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

    pub fn build(self) -> Result<SentenceEmbeddings> {
        SentenceEmbeddings::new(self)
    }
}
