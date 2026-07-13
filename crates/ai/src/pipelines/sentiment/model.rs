use crate::resources::ModelResource;

/// Checkpoints known to work with this pipeline. `ModelResource` remains available for
/// anything not listed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SentimentModel {
    DistilBertSst2,
}

impl SentimentModel {
    pub fn resource(self) -> ModelResource {
        ModelResource::new(match self {
            Self::DistilBertSst2 => "distilbert-base-uncased-finetuned-sst-2-english",
        })
    }
}

impl From<SentimentModel> for ModelResource {
    fn from(model: SentimentModel) -> Self {
        model.resource()
    }
}

impl std::fmt::Display for SentimentModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.resource())
    }
}
