use crate::models::ModelRef;
use crate::resources::ModelId;

/// Checkpoints known to work with this pipeline. `Resource` remains available for anything not
/// listed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SentenceEmbeddingsCheckpoint {
    AllMiniLmL6V2,
    AllMiniLmL12V2,
    AllMpnetBaseV2,
    AllDistilrobertaV1,
    ParaphraseMiniLmL6V2,
}

impl SentenceEmbeddingsCheckpoint {
    pub fn model(self) -> ModelRef {
        ModelRef::hub(self.id())
    }

    pub fn id(self) -> ModelId {
        let repo = match self {
            Self::AllMiniLmL6V2 => "sentence-transformers/all-MiniLM-L6-v2",
            Self::AllMiniLmL12V2 => "sentence-transformers/all-MiniLM-L12-v2",
            Self::AllMpnetBaseV2 => "sentence-transformers/all-mpnet-base-v2",
            Self::AllDistilrobertaV1 => "sentence-transformers/all-distilroberta-v1",
            Self::ParaphraseMiniLmL6V2 => "sentence-transformers/paraphrase-MiniLM-L6-v2",
        };

        repo.parse().expect("built-in model ids are valid")
    }
}

impl From<SentenceEmbeddingsCheckpoint> for ModelRef {
    fn from(checkpoint: SentenceEmbeddingsCheckpoint) -> Self {
        checkpoint.model()
    }
}

impl std::fmt::Display for SentenceEmbeddingsCheckpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}
