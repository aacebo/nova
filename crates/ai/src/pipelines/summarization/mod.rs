mod config;
mod model;
mod pipeline;

use std::sync::OnceLock;

pub use config::Config;
pub use model::SummarizationModel;
pub use pipeline::Summarization;

use crate::resources::Result;

static PIPELINE: OnceLock<Summarization> = OnceLock::new();

pub fn get() -> Result<&'static Summarization> {
    if let Some(pipeline) = PIPELINE.get() {
        return Ok(pipeline);
    }

    let pipeline = Config::default().build()?;
    Ok(PIPELINE.get_or_init(|| pipeline))
}
