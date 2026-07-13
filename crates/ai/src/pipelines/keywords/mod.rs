mod candidates;
mod config;
mod local;
mod remote;
mod scorer;
mod stopwords;

use std::sync::{Arc, LazyLock};

pub use config::Config;

use crate::models::ModelRef;
use crate::pipelines::{Cache, Key, Keywords};
use crate::resources::Result;

static PIPELINES: LazyLock<Cache<dyn Keywords>> = LazyLock::new(Cache::new);

pub fn get(model: &ModelRef, api_key: &Option<String>) -> Result<Arc<dyn Keywords>> {
    PIPELINES.get_or_build(Key::new(model, api_key), || {
        Config::default().model(model.clone()).api_key(api_key.clone()).build()
    })
}
