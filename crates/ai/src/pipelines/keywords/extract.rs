use crate::resources::Result;
use crate::types::Keyword;

/// Ranks the salient keywords of each input. Implemented by both local and remote transports.
pub trait Keywords: Send + Sync {
    fn keywords(&self, text: &[&str]) -> Result<Vec<Vec<Keyword>>>;
}
