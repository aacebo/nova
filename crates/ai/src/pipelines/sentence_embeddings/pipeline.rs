use candle_core::Device;
use tokenizers::Tokenizer;

use super::config::Config;
use crate::models::bert;
use crate::pipelines::common::Batch;
use crate::resources::{Error, Repo, Result};

pub struct SentenceEmbeddings {
    embedder: bert::Embedder,
    tokenizer: Tokenizer,
    device: Device,
}

impl SentenceEmbeddings {
    pub(super) fn new(config: Config) -> Result<Self> {
        let repo = Repo::open(config.model, config.device, config.dtype)?;
        let model: bert::Config = repo.config()?;
        let device = repo.device().clone();

        Ok(Self {
            embedder: bert::Embedder::new(repo.vars()?, &model)?,
            tokenizer: repo.tokenizer()?,
            device,
        })
    }

    pub fn encode<S: AsRef<str>>(&self, text: &[S]) -> Result<Vec<Vec<f32>>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let text: Vec<&str> = text.iter().map(AsRef::as_ref).collect();
        let encodings = self.tokenizer.encode_batch(text, true).map_err(Error::tokenize)?;
        let batch = Batch::new(encodings, &self.device)?;

        self.embedder
            .forward(&batch.ids, &batch.mask)?
            .to_vec2::<f32>()
            .map_err(Error::inference)
    }
}
