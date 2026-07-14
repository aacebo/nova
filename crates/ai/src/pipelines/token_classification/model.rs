use crate::models::ModelRef;
use crate::resources::ModelId;

/// Models known to work with this pipeline. `Resource` remains available for anything not
/// listed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenClassificationModelType {
    BertLargeConll03,
    BertBaseConll03,
}

impl TokenClassificationModelType {
    pub fn model(self) -> ModelRef {
        ModelRef::hub(self.id())
    }

    pub fn id(self) -> ModelId {
        let repo = match self {
            Self::BertLargeConll03 => "dbmdz/bert-large-cased-finetuned-conll03-english",
            Self::BertBaseConll03 => "dslim/bert-base-NER",
        };

        repo.parse().expect("built-in model ids are valid")
    }
}

impl From<TokenClassificationModelType> for ModelRef {
    fn from(model: TokenClassificationModelType) -> Self {
        model.model()
    }
}

impl std::fmt::Display for TokenClassificationModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}
