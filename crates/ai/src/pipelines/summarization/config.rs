use candle_core::{DType, Device};

use super::model::SummarizationModel;
use super::pipeline::Summarization;
use crate::resources::{self, ModelResource, Result};

pub struct Config {
    pub model: ModelResource,
    pub device: Device,
    pub dtype: DType,
    pub beams: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: SummarizationModel::BartLargeCnn.resource(),
            device: resources::default_device(),
            dtype: resources::default_dtype(),
            beams: None,
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

    /// Overrides the beam count the checkpoint specifies.
    pub fn beams(mut self, beams: usize) -> Self {
        self.beams = Some(beams);
        self
    }

    pub fn build(self) -> Result<Summarization> {
        Summarization::new(self)
    }
}
