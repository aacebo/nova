use candle_core::Device;
use tokenizers::Tokenizer;

use super::config::Config;
use crate::models::bert;
use crate::pipelines::Embed;
use crate::pipelines::common::Batch;
use crate::resources::{Error, Result};

pub struct Local {
    embedder: bert::Embedder,
    tokenizer: Tokenizer,
    device: Device,
}

impl Local {
    pub fn new(config: Config) -> Result<Self> {
        let repo = config.model.loader(config.device.clone(), config.dtype)?;
        let model: bert::Config = repo.config()?;
        let device = repo.device().clone();

        Ok(Self {
            embedder: bert::Embedder::new(repo.vars()?, &model)?,
            tokenizer: repo.tokenizer()?,
            device,
        })
    }

    pub fn encode(&self, text: &[&str]) -> Result<Vec<Vec<f32>>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let encodings = self.tokenizer.encode_batch(text.to_vec(), true).map_err(Error::tokenize)?;
        let batch = Batch::new(encodings, &self.device)?;

        self.embedder
            .forward(&batch.ids, &batch.mask)?
            .to_vec2::<f32>()
            .map_err(Error::inference)
    }
}

impl Embed for Local {
    fn embed(&self, text: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.encode(text)
    }
}
