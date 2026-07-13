use crate::pipelines::Model;
use crate::resources::ModelId;

/// Checkpoints known to work with this pipeline. `Resource` remains available for anything not
/// listed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummarizationCheckpoint {
    BartLargeCnn,
    BartLargeXsum,
}

impl SummarizationCheckpoint {
    pub fn model(self) -> Model {
        Model::hub(self.id())
    }

    pub fn id(self) -> ModelId {
        let repo = match self {
            Self::BartLargeCnn => "facebook/bart-large-cnn",
            Self::BartLargeXsum => "facebook/bart-large-xsum",
        };

        repo.parse().expect("built-in model ids are valid")
    }
}

impl From<SummarizationCheckpoint> for Model {
    fn from(checkpoint: SummarizationCheckpoint) -> Self {
        checkpoint.model()
    }
}

impl std::fmt::Display for SummarizationCheckpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}
