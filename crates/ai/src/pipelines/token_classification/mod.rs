mod aggregation;
mod checkpoint;
mod config;
mod extract;
mod local;
mod pii;
mod remote;

use std::sync::{Arc, LazyLock};

pub use checkpoint::TokenClassificationCheckpoint;
pub use config::Config;
pub use extract::Extract;

use crate::pipelines::cache::Cache;
use crate::pipelines::{Key, Model};
use crate::resources::Result;

static PIPELINES: LazyLock<Cache<dyn Extract>> = LazyLock::new(Cache::new);

pub fn get(model: &Model, api_key: &Option<String>) -> Result<Arc<dyn Extract>> {
    PIPELINES.get_or_build(Key::new(model, api_key), || {
        Config::default().model(model.clone()).api_key(api_key.clone()).build()
    })
}
