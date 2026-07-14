use std::sync::Mutex;

use candle_core::{Device, Tensor};
use tokenizers::Tokenizer;

use super::config::Config;
use crate::models::bart;
use crate::pipelines::{Summarize, generate};
use crate::resources::{Error, Result};

pub struct Local {
    model: Mutex<bart::Bart>,
    generation: generate::Config,
    tokenizer: Tokenizer,
    max_position_embeddings: usize,
    device: Device,
}

impl Local {
    pub fn new(config: Config) -> Result<Self> {
        let repo = config.model.loader(config.device.clone(), config.dtype)?;
        let model: bart::Config = repo.config()?;
        let device = repo.device().clone();

        let mut generation = generate::Config::from(&model);

        if let Some(beams) = config.beams {
            generation = generation.beams(beams);
        }

        Ok(Self {
            model: Mutex::new(bart::Bart::new(&model, repo.vars()?).map_err(Error::load)?),
            generation,
            tokenizer: repo.tokenizer()?,
            max_position_embeddings: model.max_position_embeddings,
            device,
        })
    }

    pub fn eval(&self, text: &[&str]) -> Result<Vec<String>> {
        text.iter().map(|text| self.eval_one(text)).collect()
    }

    pub fn eval_one(&self, text: &str) -> Result<String> {
        let encoding = self.tokenizer.encode(text, true).map_err(Error::tokenize)?;
        let ids = encoding.get_ids();

        // The learned positional embeddings only cover `max_position_embeddings`.
        let ids = &ids[..ids.len().min(self.max_position_embeddings)];
        let input = Tensor::from_slice(ids, (1, ids.len()), &self.device).map_err(Error::inference)?;

        // Generation mutates the KV cache, so the model is serialized behind a mutex.
        let mut model = self
            .model
            .lock()
            .map_err(|_| Error::Inference("summarization model lock poisoned".to_string()))?;

        let tokens = generate::run(&mut model, &self.generation, &input, &self.device)?;

        self.tokenizer
            .decode(&tokens, true)
            .map(|summary| summary.trim().to_string())
            .map_err(Error::tokenize)
    }
}

impl Summarize for Local {
    fn summarize(&self, text: &[&str]) -> Result<Vec<String>> {
        self.eval(text)
    }
}
