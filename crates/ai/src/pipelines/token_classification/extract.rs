use crate::resources::Result;
use crate::types::Entity;

/// Extracts entities from each input. Returns *decoded* entities, not raw per-token tags: the
/// local transport runs IOB/BIOES decoding internally, and a remote transport is assumed to have
/// done the equivalent. This is what lets either back the same routine.
pub trait Extract: Send + Sync {
    /// Named entities (IOB1).
    fn entities(&self, text: &[&str]) -> Result<Vec<Vec<Entity>>>;

    /// Personally identifying information (BIOES, with adjacency merging).
    /// `min_score` is applied *before* adjacent entities are merged, so a weak neighbour
    /// cannot drag a strong entity below the threshold (or be resurrected by one).
    fn pii(&self, text: &[&str], min_score: f64) -> Result<Vec<Vec<Entity>>>;
}
