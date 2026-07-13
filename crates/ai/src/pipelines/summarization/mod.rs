mod checkpoint;
mod config;
mod local;
mod remote;
mod summarize;

use std::sync::{Arc, LazyLock};

pub use checkpoint::SummarizationCheckpoint;
pub use config::Config;
pub use summarize::Summarize;

use crate::pipelines::cache::Cache;
use crate::pipelines::{Key, Model};
use crate::resources::Result;

static PIPELINES: LazyLock<Cache<dyn Summarize>> = LazyLock::new(Cache::new);

pub fn get(model: &Model, api_key: &Option<String>) -> Result<Arc<dyn Summarize>> {
    PIPELINES.get_or_build(Key::new(model, api_key), || {
        Config::default().model(model.clone()).api_key(api_key.clone()).build()
    })
}
