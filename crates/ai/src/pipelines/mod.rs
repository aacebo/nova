mod anchor;
mod cache;
mod common;

pub mod generation;
pub mod keywords;
pub mod sentence_embeddings;
pub mod sentiment;
pub mod summarization;
pub mod token_classification;

pub use cache::{Cache, Key};

use crate::resources::Result;
use crate::types::{Entity, Keyword, Sentiment};

pub trait Embed: Send + Sync {
    fn embed(&self, text: &[&str]) -> Result<Vec<Vec<f32>>>;
}

pub trait Classify: Send + Sync {
    fn classify(&self, text: &[&str]) -> Result<Vec<Sentiment>>;
}

pub trait Keywords: Send + Sync {
    fn keywords(&self, text: &[&str]) -> Result<Vec<Vec<Keyword>>>;
}

pub trait Extract: Send + Sync {
    fn entities(&self, text: &[&str]) -> Result<Vec<Vec<Entity>>>;

    /// `min_score` is applied *before* adjacent entities are merged, so a weak neighbour cannot
    /// drag a strong entity below the threshold (or be resurrected by one).
    fn pii(&self, text: &[&str], min_score: f64) -> Result<Vec<Vec<Entity>>>;
}

pub trait Summarize: Send + Sync {
    fn summarize(&self, text: &[&str]) -> Result<Vec<String>>;
}
