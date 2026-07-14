use candle_core::{DType, Device};

use super::{Keywords, local, remote};
use crate::clients::openai::OpenAI;
use crate::models::ModelRef;
use crate::pipelines::embeddings::SentenceEmbeddingsModelType;
use crate::resources::{Provider, Result};

pub struct Config {
    pub model: ModelRef,
    pub api_key: Option<String>,
    pub device: Device,
    pub dtype: DType,
    pub top_n: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: SentenceEmbeddingsModelType::AllMiniLmL6V2.model(),
            api_key: None,
            device: Device::Cpu,
            dtype: DType::F32,
            top_n: 5,
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

    pub fn top_n(mut self, top_n: usize) -> Self {
        self.top_n = top_n;
        self
    }

    pub fn build(self) -> Result<std::sync::Arc<dyn Keywords>> {
        match &self.model {
            ModelRef::Remote { provider, id, base_url } => match provider {
                Provider::OpenAI => {
                    let client = OpenAI::new(id.clone(), base_url.clone(), self.api_key.clone());
                    Ok(std::sync::Arc::new(remote::Remote::new(client, self.top_n)))
                }
            },
            _ => Ok(std::sync::Arc::new(local::Local::new(self)?)),
        }
    }
}
