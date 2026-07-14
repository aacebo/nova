use crate::models::ModelRef;
use crate::resources::ModelId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SentimentModelType {
    DistilBertSst2,
}

impl SentimentModelType {
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

impl From<SentimentModelType> for ModelRef {
    fn from(model: SentimentModelType) -> Self {
        model.model()
    }
}

impl std::fmt::Display for SentimentModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}
