use candle_core::{DType, Device};

use super::model::SummarizationModelType;
use super::{Summarize, local, remote};
use crate::clients::openai::OpenAI;
use crate::models::ModelRef;
use crate::resources::{Provider, Result};

pub struct Config {
    pub model: ModelRef,
    pub api_key: Option<String>,
    pub device: Device,
    pub dtype: DType,
    pub beams: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: SummarizationModelType::BartLargeCnn.model(),
            api_key: None,
            device: Device::Cpu,
            dtype: DType::F32,
            beams: None,
        }
    }
}

impl Config {
    pub fn model(mut self, model: ModelRef) -> Self {
        self.model = model;
        self
    }

    pub fn api_key(mut self, api_key: Option<String>) -> Self {
        self.api_key = api_key;
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

    pub fn beams(mut self, beams: usize) -> Self {
        self.beams = Some(beams);
        self
    }

    pub fn build(self) -> Result<std::sync::Arc<dyn Summarize>> {
        match &self.model {
            ModelRef::Remote { provider, id, base_url } => match provider {
                Provider::OpenAI => {
                    let client = OpenAI::new(id.clone(), base_url.clone(), self.api_key.clone());
                    Ok(std::sync::Arc::new(remote::Remote::new(client)))
                }
            },
            _ => Ok(std::sync::Arc::new(local::Local::new(self)?)),
        }
    }
}
