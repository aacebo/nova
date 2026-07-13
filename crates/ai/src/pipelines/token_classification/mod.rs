mod aggregation;
mod checkpoint;
mod config;
mod local;
mod pii;
mod remote;

use std::sync::{Arc, LazyLock};

pub use checkpoint::TokenClassificationCheckpoint;
pub use config::Config;

use crate::models::ModelRef;
use crate::pipelines::{Cache, Extract, Key};
use crate::resources::Result;

static PIPELINES: LazyLock<Cache<dyn Extract>> = LazyLock::new(Cache::new);

pub fn get(model: &ModelRef, api_key: &Option<String>) -> Result<Arc<dyn Extract>> {
    PIPELINES.get_or_build(Key::new(model, api_key), || {
        Config::default().model(model.clone()).api_key(api_key.clone()).build()
    })
}
