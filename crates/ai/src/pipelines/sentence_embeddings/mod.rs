mod checkpoint;
mod config;
mod local;
mod remote;

use std::sync::{Arc, LazyLock};

pub use checkpoint::SentenceEmbeddingsCheckpoint;
pub use config::Config;

use crate::models::ModelRef;
use crate::pipelines::{Cache, Embed, Key};
use crate::resources::Result;

static PIPELINES: LazyLock<Cache<dyn Embed>> = LazyLock::new(Cache::new);

pub fn get(model: &ModelRef, api_key: &Option<String>) -> Result<Arc<dyn Embed>> {
    PIPELINES.get_or_build(Key::new(model, api_key), || {
        Config::default().model(model.clone()).api_key(api_key.clone()).build()
    })
}
