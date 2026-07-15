mod cache;
mod routines;

pub mod common;
pub mod generate;

pub use cache::{Cache, Key};
pub use common::Batch;
use nova_core::{Args, FromArgs};
pub use routines::{embeddings, entities, keywords, pii, sentiment, summarize};

use crate::models::ModelRef;
use crate::resources::{ModelId, Provider, Result, Uri};

type RoutineResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct ModelArgs {
    provider: Option<String>,
    model: Option<String>,
    base_url: Option<String>,
}

impl ModelArgs {
    pub fn resolve(self, default: ModelRef) -> RoutineResult<ModelRef> {
        let Some(provider) = self.provider else {
            let Some(model) = self.model else {
                return Ok(default);
            };

            return Ok(match is_uri(&model) {
                true => ModelRef::local(model.parse::<Uri>()?),
                false => ModelRef::hub(model.parse::<ModelId>()?),
            });
        };

        let provider: Provider = provider.parse()?;
        let id: ModelId = self
            .model
            .ok_or_else(|| nova_core::Error::message("model is required when provider is set"))?
            .parse()?;

        Ok(ModelRef::remote(provider, id).base_url(self.base_url))
    }
}

impl FromArgs for ModelArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &Args) -> RoutineResult<Self> {
        Ok(Self {
            provider: string(args, "provider")?,
            model: string(args, "model")?,
            base_url: string(args, "base_url")?,
        })
    }
}

pub struct TextArgs {
    pub text: Vec<String>,
    pub model: ModelArgs,
    pub api_key: Option<String>,
}

impl FromArgs for TextArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &Args) -> RoutineResult<Self> {
        Ok(Self {
            text: text(args)?,
            model: ModelArgs::from_args(args)?,
            api_key: string(args, "api_key")?,
        })
    }
}

pub struct ScoredArgs {
    pub text: Vec<String>,
    pub min_score: f64,
    pub model: ModelArgs,
    pub api_key: Option<String>,
}

impl FromArgs for ScoredArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &Args) -> RoutineResult<Self> {
        Ok(Self {
            text: text(args)?,
            min_score: min_score(args)?,
            model: ModelArgs::from_args(args)?,
            api_key: string(args, "api_key")?,
        })
    }
}

pub fn borrow(text: &[String]) -> Vec<&str> {
    text.iter().map(String::as_str).collect()
}

fn text(args: &Args) -> RoutineResult<Vec<String>> {
    let value = args.at(0);

    if let Some(text) = value.as_str() {
        return Ok(vec![text.to_string()]);
    }

    let map = value
        .as_map()
        .ok_or(nova_core::Error::message("text must be a string or list of strings"))?;

    let mut out = Vec::new();

    for (_, item) in map.iter() {
        let item = item
            .as_str()
            .ok_or(nova_core::Error::message("text must be a string or list of strings"))?;

        out.push(item.to_string());
    }

    Ok(out)
}

fn min_score(args: &Args) -> RoutineResult<f64> {
    let value = args.key("min_score");

    if value.is_null() || value.is_undefined() {
        return Ok(0.0);
    }

    value
        .to_f64()
        .ok_or_else(|| nova_core::Error::message("min_score must be a number").into())
}

fn is_uri(model: &str) -> bool {
    let scheme = model.starts_with("file://") || model.starts_with("http://") || model.starts_with("https://");

    scheme || std::path::Path::new(model).is_dir()
}

fn string(args: &Args, key: &str) -> RoutineResult<Option<String>> {
    let value = args.key(key);

    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }

    value
        .as_str()
        .map(|value| Some(value.to_string()))
        .ok_or_else(|| nova_core::Error::message(format!("{key} must be a string")).into())
}

/// Default models, one per capability. A model that cannot serve the routine you called is an
/// error -- the empty cells of the capability matrix.
pub(crate) mod defaults {
    use crate::models::ModelRef;
    use crate::resources::ModelId;

    fn hub(repo: &str) -> ModelRef {
        ModelRef::hub(repo.parse::<ModelId>().expect("built-in model ids are valid"))
    }

    pub fn embed() -> ModelRef {
        hub("sentence-transformers/all-MiniLM-L12-v2")
    }

    pub fn keywords() -> ModelRef {
        hub("sentence-transformers/all-MiniLM-L6-v2")
    }

    pub fn classify() -> ModelRef {
        hub("distilbert-base-uncased-finetuned-sst-2-english")
    }

    pub fn token_classify() -> ModelRef {
        hub("dbmdz/bert-large-cased-finetuned-conll03-english")
    }

    pub fn generate() -> ModelRef {
        hub("facebook/bart-large-cnn")
    }
}

static MODELS: std::sync::LazyLock<Cache<crate::models::Loaded>> = std::sync::LazyLock::new(Cache::new);

/// One cache of loaded models, keyed by `(model, api_key)` -- not one per capability. A model used
/// for two routines now loads its weights once.
pub fn load(model: &ModelRef, api_key: &Option<String>) -> Result<std::sync::Arc<crate::models::Loaded>> {
    use candle_core::{DType, Device};

    MODELS.get_or_build(Key::new(model, api_key), || {
        Ok(std::sync::Arc::new(crate::models::Loaded::new(
            model,
            api_key,
            Device::Cpu,
            DType::F32,
        )?))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(model: &str) -> ModelArgs {
        ModelArgs {
            provider: None,
            model: Some(model.to_string()),
            base_url: None,
        }
    }

    /// The defaults differ per capability, so a routine left to itself embeds with one model and
    /// extracts keywords with another. That is what makes the next test worth writing.
    #[test]
    fn the_capability_defaults_are_different_models() {
        assert_ne!(defaults::embed(), defaults::keywords());
    }

    /// An explicit `model=` overrides the per-capability default, so `ai.embeddings` and
    /// `ai.keywords` on one model resolve to one cache key -- and the weights load once. Under a
    /// cache keyed by capability rather than by model, these keys differed and the weights loaded
    /// twice.
    #[test]
    fn two_routines_on_one_model_share_a_cache_key() {
        let name = "sentence-transformers/all-MiniLM-L12-v2";

        // `embeddings` and `keywords` resolve the same args against their own default.
        let embed = args(name).resolve(defaults::embed()).unwrap();
        let keywords = args(name).resolve(defaults::keywords()).unwrap();

        assert_eq!(embed, keywords);
        assert_eq!(Key::new(&embed, &None), Key::new(&keywords, &None));
    }

    /// Without `model=`, each routine falls back to its own default -- so they do NOT share.
    #[test]
    fn routines_left_on_their_defaults_do_not_share_a_cache_key() {
        let embed = ModelArgs {
            provider: None,
            model: None,
            base_url: None,
        }
        .resolve(defaults::embed())
        .unwrap();

        let keywords = ModelArgs {
            provider: None,
            model: None,
            base_url: None,
        }
        .resolve(defaults::keywords())
        .unwrap();

        assert_ne!(Key::new(&embed, &None), Key::new(&keywords, &None));
    }
}
