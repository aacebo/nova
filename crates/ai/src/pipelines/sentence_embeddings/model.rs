use crate::resources::ModelResource;

/// Checkpoints known to work with this pipeline. `ModelResource` remains available for
/// anything not listed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SentenceEmbeddingsModel {
    AllMiniLmL6V2,
    AllMiniLmL12V2,
    AllMpnetBaseV2,
    AllDistilrobertaV1,
    ParaphraseMiniLmL6V2,
}

impl SentenceEmbeddingsModel {
    pub fn resource(self) -> ModelResource {
        ModelResource::new(match self {
            Self::AllMiniLmL6V2 => "sentence-transformers/all-MiniLM-L6-v2",
            Self::AllMiniLmL12V2 => "sentence-transformers/all-MiniLM-L12-v2",
            Self::AllMpnetBaseV2 => "sentence-transformers/all-mpnet-base-v2",
            Self::AllDistilrobertaV1 => "sentence-transformers/all-distilroberta-v1",
            Self::ParaphraseMiniLmL6V2 => "sentence-transformers/paraphrase-MiniLM-L6-v2",
        })
    }
}

impl From<SentenceEmbeddingsModel> for ModelResource {
    fn from(model: SentenceEmbeddingsModel) -> Self {
        model.resource()
    }
}

impl std::fmt::Display for SentenceEmbeddingsModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.resource())
    }
}
