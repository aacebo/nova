use crate::resources::Result;

/// Produces an abstractive summary per input. Implemented by both local and remote transports.
pub trait Summarize: Send + Sync {
    fn summarize(&self, text: &[&str]) -> Result<Vec<String>>;
}
