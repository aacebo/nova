mod anchor;
mod cache;
mod common;

pub mod embeddings;
pub mod generate;
pub mod keywords;
pub mod sentiment;
pub mod summarize;
pub mod token_classification;

pub use cache::{Cache, Key};
use nova_core::{Args, FromArgs};

use crate::models::ModelRef;
use crate::resources::{ModelId, Provider, Result, Uri};
use crate::types::{Entity, Keyword, Sentiment};

type RoutineResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Embed: Send + Sync {
    fn embed(&self, text: &[&str]) -> Result<Vec<Vec<f32>>>;
}

pub trait Classify: Send + Sync {
    fn classify(&self, text: &[&str]) -> Result<Vec<Sentiment>>;
}

pub trait Keywords: Send + Sync {
    fn keywords(&self, text: &[&str]) -> Result<Vec<Vec<Keyword>>>;
}

pub trait Extract: Send + Sync {
    fn entities(&self, text: &[&str]) -> Result<Vec<Vec<Entity>>>;
    fn pii(&self, text: &[&str], min_score: f64) -> Result<Vec<Vec<Entity>>>;
}

pub trait Summarize: Send + Sync {
    fn summarize(&self, text: &[&str]) -> Result<Vec<String>>;
}

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

    fn from_args(args: &Args<'_>) -> RoutineResult<Self> {
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

    fn from_args(args: &Args<'_>) -> RoutineResult<Self> {
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

    fn from_args(args: &Args<'_>) -> RoutineResult<Self> {
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

    let mut out = Vec::new();

    for item in value.try_iter()? {
        let item = item
            .as_str()
            .ok_or(nova_core::Error::message("text must be a string or list of strings"))?;

        out.push(item.to_string());
    }

    Ok(out)
}

fn min_score(args: &Args) -> RoutineResult<f64> {
    match args.key("min_score") {
        v if v.is_undefined() || v.is_none() => Ok(0.0),
        v => f64::try_from(v).map_err(|_| nova_core::Error::message("min_score must be a number").into()),
    }
}

fn is_uri(model: &str) -> bool {
    let scheme = model.starts_with("file://") || model.starts_with("http://") || model.starts_with("https://");

    scheme || std::path::Path::new(model).is_dir()
}

fn string(args: &Args, key: &str) -> RoutineResult<Option<String>> {
    match args.key(key) {
        v if v.is_undefined() || v.is_none() => Ok(None),
        v => v
            .as_str()
            .map(|value| Some(value.to_string()))
            .ok_or_else(|| nova_core::Error::message(format!("{key} must be a string")).into()),
    }
}
