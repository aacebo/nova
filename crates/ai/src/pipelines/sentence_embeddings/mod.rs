mod checkpoint;
mod config;
mod embed;
mod local;
mod remote;

use std::sync::{Arc, LazyLock};

pub use checkpoint::SentenceEmbeddingsCheckpoint;
pub use config::Config;
pub use embed::Embed;

use crate::pipelines::cache::Cache;
use crate::pipelines::{Key, Model};
use crate::resources::Result;

static PIPELINES: LazyLock<Cache<dyn Embed>> = LazyLock::new(Cache::new);

pub fn get(model: &Model, api_key: &Option<String>) -> Result<Arc<dyn Embed>> {
    PIPELINES.get_or_build(Key::new(model, api_key), || {
        Config::default().model(model.clone()).api_key(api_key.clone()).build()
    })
}
