pub mod bart;
pub mod bert;
pub mod distilbert;

use std::str::FromStr;
use std::sync::Arc;

use candle_core::{DType, Device};

use crate::clients::fs::FileSystem;
use crate::clients::hf::HuggingFace;
use crate::clients::http::Http;
use crate::resources::{Error, Loader, ModelId, Provider, Repository, Resource, Result, Uri};

pub trait Forward: Send + Sync {
    type Input;
    type Output;

    fn forward(&self, input: Self::Input) -> Result<Self::Output>;
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Architecture {
    Bart,
    Bert,
    DistilBert,
    Roberta,
    Deberta,
    Gpt2,
    Llama,
    Mistral,
    T5,
    #[default]
    #[serde(other)]
    Unknown,
}

impl Architecture {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bart => "bart",
            Self::Bert => "bert",
            Self::DistilBert => "distilbert",
            Self::Roberta => "roberta",
            Self::Deberta => "deberta",
            Self::Gpt2 => "gpt2",
            Self::Llama => "llama",
            Self::Mistral => "mistral",
            Self::T5 => "t5",
            Self::Unknown => "??",
        }
    }
}

impl FromStr for Architecture {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        Ok(match value.to_lowercase().as_str() {
            "bart" => Self::Bart,
            "bert" => Self::Bert,
            "distilbert" => Self::DistilBert,
            "roberta" => Self::Roberta,
            "deberta" => Self::Deberta,
            "gpt2" => Self::Gpt2,
            "llama" => Self::Llama,
            "mistral" => Self::Mistral,
            "t5" => Self::T5,
            _ => Self::Unknown,
        })
    }
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelRef {
    Hub(ModelId),
    Local(Resource),
    Remote {
        provider: Provider,
        id: ModelId,
        base_url: Option<String>,
    },
}

impl ModelRef {
    pub fn hub(id: ModelId) -> Self {
        Self::Hub(id)
    }

    pub fn local(uri: Uri) -> Self {
        Self::Local(Resource::new(uri))
    }

    pub fn remote(provider: Provider, id: ModelId) -> Self {
        Self::Remote {
            provider,
            id,
            base_url: None,
        }
    }

    pub fn base_url(mut self, url: Option<String>) -> Self {
        if let Self::Remote { base_url, .. } = &mut self {
            *base_url = url;
        }

        self
    }

    pub fn is_remote(&self) -> bool {
        matches!(self, Self::Remote { .. })
    }

    pub fn loader(&self, device: Device, dtype: DType) -> Result<Loader> {
        Ok(Loader::new(self.repository()?, device, dtype))
    }

    pub fn repository(&self) -> Result<Arc<dyn Repository>> {
        match self {
            Self::Hub(id) => Ok(Arc::new(HuggingFace::new(id)?)),
            Self::Local(resource) => match &resource.uri {
                Uri::Local(path) => Ok(Arc::new(FileSystem::new(path))),
                Uri::Http(_) => Ok(Arc::new(Http::new(resource.uri.clone()))),
            },
            Self::Remote { id, .. } => Err(Error::Load(format!("{id} is a remote model and has no weights"))),
        }
    }
}

impl std::fmt::Display for ModelRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hub(id) => write!(f, "{id}"),
            Self::Local(resource) => write!(f, "{resource}"),
            Self::Remote { provider, id, .. } => write!(f, "{provider}:{id}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_config_json_model_type() {
        assert_eq!("bert".parse::<Architecture>().unwrap(), Architecture::Bert);
        assert_eq!("distilbert".parse::<Architecture>().unwrap(), Architecture::DistilBert);
    }

    /// An unrecognised architecture must not be an error -- config.json may name anything.
    #[test]
    fn an_unknown_architecture_is_not_an_error() {
        assert_eq!("mamba".parse::<Architecture>().unwrap(), Architecture::Unknown);
        assert_eq!(
            serde_json::from_str::<Architecture>("\"mamba\"").unwrap(),
            Architecture::Unknown
        );
    }

    #[test]
    fn serde_round_trips() {
        assert_eq!(serde_json::to_string(&Architecture::Bert).unwrap(), "\"bert\"");
        assert_eq!(serde_json::from_str::<Architecture>("\"bart\"").unwrap(), Architecture::Bart);
    }
}
