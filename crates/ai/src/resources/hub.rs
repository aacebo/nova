use candle_core::{DType, Device};
use candle_nn::VarBuilder;
use hf_hub::api::sync::{ApiBuilder, ApiRepo};
use tokenizers::Tokenizer;

use super::error::{Error, Result};
use super::{cache, tokenizer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelResource(&'static str);

impl ModelResource {
    pub const fn new(repo: &'static str) -> Self {
        Self(repo)
    }

    pub fn repo(&self) -> &str {
        self.0
    }
}

impl std::fmt::Display for ModelResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A resolved checkpoint: downloads (and caches) the files a model needs.
pub struct Repo {
    inner: ApiRepo,
    device: Device,
    dtype: DType,
}

impl Repo {
    pub fn open(model: ModelResource, device: Device, dtype: DType) -> Result<Self> {
        let mut builder = ApiBuilder::new();

        if let Some(dir) = cache::dir() {
            builder = builder.with_cache_dir(dir);
        }

        let api = builder.build().map_err(Error::load)?;

        Ok(Self {
            inner: api.model(model.repo().to_string()),
            device,
            dtype,
        })
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn config<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        self.json("config.json")
    }

    pub fn tokenizer(&self) -> Result<Tokenizer> {
        if let Ok(path) = self.inner.get("tokenizer.json") {
            return tokenizer::from_file(path);
        }

        let path = self.inner.get("vocab.txt").map_err(Error::load)?;
        tokenizer::from_vocab(path, self.lowercase()?)
    }

    pub fn vars(&self) -> Result<VarBuilder<'static>> {
        let path = self.inner.get("model.safetensors").map_err(Error::load)?;

        unsafe { VarBuilder::from_mmaped_safetensors(&[path], self.dtype, &self.device) }.map_err(Error::load)
    }

    fn json<T: serde::de::DeserializeOwned>(&self, file: &str) -> Result<T> {
        let path = self.inner.get(file).map_err(Error::load)?;
        let bytes = std::fs::read(path).map_err(Error::load)?;

        serde_json::from_slice(&bytes).map_err(Error::load)
    }

    /// Model-specific: SST-2 is uncased, CoNLL-03 is cased. Lowercasing a cased model wrecks it.
    fn lowercase(&self) -> Result<bool> {
        #[derive(serde::Deserialize)]
        struct Config {
            #[serde(default)]
            do_lower_case: bool,
        }

        if self.inner.get("tokenizer_config.json").is_err() {
            return Ok(false);
        }

        let config: Config = self.json("tokenizer_config.json")?;
        Ok(config.do_lower_case)
    }
}
