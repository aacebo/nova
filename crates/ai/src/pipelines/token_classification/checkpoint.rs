use crate::pipelines::Model;
use crate::resources::ModelId;

/// Checkpoints known to work with this pipeline. `Resource` remains available for anything not
/// listed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenClassificationCheckpoint {
    BertLargeConll03,
    BertBaseConll03,
}

impl TokenClassificationCheckpoint {
    pub fn model(self) -> Model {
        Model::hub(self.id())
    }

    pub fn id(self) -> ModelId {
        let repo = match self {
            Self::BertLargeConll03 => "dbmdz/bert-large-cased-finetuned-conll03-english",
            Self::BertBaseConll03 => "dslim/bert-base-NER",
        };

        repo.parse().expect("built-in model ids are valid")
    }
}

impl From<TokenClassificationCheckpoint> for Model {
    fn from(checkpoint: TokenClassificationCheckpoint) -> Self {
        checkpoint.model()
    }
}

impl std::fmt::Display for TokenClassificationCheckpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}
