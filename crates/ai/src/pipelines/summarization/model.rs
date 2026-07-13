use crate::resources::ModelResource;

/// Checkpoints known to work with this pipeline. `ModelResource` remains available for
/// anything not listed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummarizationModel {
    BartLargeCnn,
    BartLargeXsum,
}

impl SummarizationModel {
    pub fn resource(self) -> ModelResource {
        ModelResource::new(match self {
            Self::BartLargeCnn => "facebook/bart-large-cnn",
            Self::BartLargeXsum => "facebook/bart-large-xsum",
        })
    }
}

impl From<SummarizationModel> for ModelResource {
    fn from(model: SummarizationModel) -> Self {
        model.resource()
    }
}

impl std::fmt::Display for SummarizationModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.resource())
    }
}
