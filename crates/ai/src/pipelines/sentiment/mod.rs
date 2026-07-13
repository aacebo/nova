mod checkpoint;
mod classify;
mod config;
mod local;
mod remote;

use std::sync::{Arc, LazyLock};

pub use checkpoint::SentimentCheckpoint;
pub use classify::Classify;
pub use config::Config;

use crate::pipelines::cache::Cache;
use crate::pipelines::{Key, Model};
use crate::resources::Result;

static PIPELINES: LazyLock<Cache<dyn Classify>> = LazyLock::new(Cache::new);

pub fn get(model: &Model, api_key: &Option<String>) -> Result<Arc<dyn Classify>> {
    PIPELINES.get_or_build(Key::new(model, api_key), || {
        Config::default().model(model.clone()).api_key(api_key.clone()).build()
    })
}
