use crate::models::ModelRef;
use crate::resources::ModelId;

/// Checkpoints known to work with this pipeline. `Resource` remains available for anything not
/// listed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SentimentCheckpoint {
    DistilBertSst2,
}

impl SentimentCheckpoint {
    pub fn model(self) -> ModelRef {
        ModelRef::hub(self.id())
    }

    pub fn id(self) -> ModelId {
        let repo = match self {
            Self::DistilBertSst2 => "distilbert-base-uncased-finetuned-sst-2-english",
        };

        repo.parse().expect("built-in model ids are valid")
    }
}

impl From<SentimentCheckpoint> for ModelRef {
    fn from(checkpoint: SentimentCheckpoint) -> Self {
        checkpoint.model()
    }
}

impl std::fmt::Display for SentimentCheckpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}
