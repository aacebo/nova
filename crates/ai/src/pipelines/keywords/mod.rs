mod candidates;
mod config;
mod extract;
mod local;
mod remote;
mod scorer;
mod stopwords;

use std::sync::{Arc, LazyLock};

pub use config::Config;
pub use extract::Keywords;

use crate::pipelines::cache::Cache;
use crate::pipelines::{Key, Model};
use crate::resources::Result;

static PIPELINES: LazyLock<Cache<dyn Keywords>> = LazyLock::new(Cache::new);

pub fn get(model: &Model, api_key: &Option<String>) -> Result<Arc<dyn Keywords>> {
    PIPELINES.get_or_build(Key::new(model, api_key), || {
        Config::default().model(model.clone()).api_key(api_key.clone()).build()
    })
}
