mod candidates;
mod config;
mod pipeline;
mod scorer;
mod stopwords;

use std::sync::OnceLock;

pub use config::Config;
pub use pipeline::Keywords;

use crate::resources::Result;

static PIPELINE: OnceLock<Keywords> = OnceLock::new();

pub fn get() -> Result<&'static Keywords> {
    if let Some(pipeline) = PIPELINE.get() {
        return Ok(pipeline);
    }

    let pipeline = Config::default().build()?;
    Ok(PIPELINE.get_or_init(|| pipeline))
}
