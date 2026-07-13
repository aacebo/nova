mod config;
mod model;
mod pipeline;

use std::sync::OnceLock;

pub use config::Config;
pub use model::SentimentModel;
pub use pipeline::Sentiment;

use crate::resources::Result;

static PIPELINE: OnceLock<Sentiment> = OnceLock::new();

pub fn get() -> Result<&'static Sentiment> {
    if let Some(pipeline) = PIPELINE.get() {
        return Ok(pipeline);
    }

    let pipeline = Config::default().build()?;
    Ok(PIPELINE.get_or_init(|| pipeline))
}
