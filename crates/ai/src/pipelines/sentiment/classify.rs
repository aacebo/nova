use crate::resources::Result;
use crate::types::Sentiment;

/// Classifies the polarity of each input. Implemented by both local and remote transports.
pub trait Classify: Send + Sync {
    fn classify(&self, text: &[&str]) -> Result<Vec<Sentiment>>;
}
