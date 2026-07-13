mod config;
mod model;
mod pipeline;

use std::sync::OnceLock;

pub use config::Config;
pub use model::SentenceEmbeddingsModel;
pub use pipeline::SentenceEmbeddings;

use crate::resources::Result;

static PIPELINE: OnceLock<SentenceEmbeddings> = OnceLock::new();

pub fn get() -> Result<&'static SentenceEmbeddings> {
    if let Some(pipeline) = PIPELINE.get() {
        return Ok(pipeline);
    }

    let pipeline = Config::default().build()?;
    Ok(PIPELINE.get_or_init(|| pipeline))
}
