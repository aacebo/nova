use crate::models::ModelRef;
use crate::resources::ModelId;

/// Models known to work with this pipeline. `Resource` remains available for anything not
/// listed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummarizationModelType {
    BartLargeCnn,
    BartLargeXsum,
}

impl SummarizationModelType {
    pub fn model(self) -> ModelRef {
        ModelRef::hub(self.id())
    }

    pub fn id(self) -> ModelId {
        let repo = match self {
            Self::BartLargeCnn => "facebook/bart-large-cnn",
            Self::BartLargeXsum => "facebook/bart-large-xsum",
        };

        repo.parse().expect("built-in model ids are valid")
    }
}

impl From<SummarizationModelType> for ModelRef {
    fn from(model: SummarizationModelType) -> Self {
        model.model()
    }
}

impl std::fmt::Display for SummarizationModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}
