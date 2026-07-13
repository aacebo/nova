mod aggregation;
mod config;
mod model;
mod pipeline;

use std::sync::OnceLock;

pub use config::Config;
pub use model::TokenClassificationModel;
pub use pipeline::TokenClassification;

use crate::resources::Result;

static PIPELINE: OnceLock<TokenClassification> = OnceLock::new();

/// NER and PII run the same checkpoint, so they share one pipeline rather than loading
/// BERT-large twice.
pub fn get() -> Result<&'static TokenClassification> {
    if let Some(pipeline) = PIPELINE.get() {
        return Ok(pipeline);
    }

    let pipeline = Config::default().build()?;
    Ok(PIPELINE.get_or_init(|| pipeline))
}
