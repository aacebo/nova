use crate::resources::Result;

/// Produces a sentence vector per input. Implemented by both local and remote transports.
pub trait Embed: Send + Sync {
    fn embed(&self, text: &[&str]) -> Result<Vec<Vec<f32>>>;
}
