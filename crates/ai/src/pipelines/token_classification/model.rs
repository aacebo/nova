use crate::resources::ModelResource;

/// Checkpoints known to work with this pipeline. `ModelResource` remains available for
/// anything not listed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenClassificationModel {
    BertLargeConll03,
    BertBaseConll03,
}

impl TokenClassificationModel {
    pub fn resource(self) -> ModelResource {
        ModelResource::new(match self {
            Self::BertLargeConll03 => "dbmdz/bert-large-cased-finetuned-conll03-english",
            Self::BertBaseConll03 => "dslim/bert-base-NER",
        })
    }
}

impl From<TokenClassificationModel> for ModelResource {
    fn from(model: TokenClassificationModel) -> Self {
        model.resource()
    }
}

impl std::fmt::Display for TokenClassificationModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.resource())
    }
}
