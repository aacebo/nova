use nova_core::Args;

use crate::pipelines::Model;
use crate::resources::{ModelId, Provider, Uri};

type Error = Box<dyn std::error::Error>;

pub fn text(args: &Args) -> Result<Vec<String>, Error> {
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

pub fn borrow(text: &[String]) -> Vec<&str> {
    text.iter().map(String::as_str).collect()
}

pub fn min_score(args: &Args) -> Result<f64, Error> {
    match args.key("min_score") {
        v if v.is_undefined() || v.is_none() => Ok(0.0),
        v => f64::try_from(v).map_err(|_| nova_core::Error::message("min_score must be a number").into()),
    }
}

pub fn api_key(args: &Args) -> Result<Option<String>, Error> {
    string(args, "api_key")
}

/// `provider=` selects a hosted API. Otherwise `model=` is parsed by shape: a path or url is a
/// local weight source, anything else is a hub id.
pub fn model(args: &Args, default: Model) -> Result<Model, Error> {
    let provider = string(args, "provider")?;
    let model = string(args, "model")?;

    let Some(provider) = provider else {
        let Some(model) = model else {
            return Ok(default);
        };

        return Ok(match is_uri(&model) {
            true => Model::local(model.parse::<Uri>()?),
            false => Model::hub(model.parse::<ModelId>()?),
        });
    };

    let provider: Provider = provider.parse()?;
    let id: ModelId = model
        .ok_or_else(|| nova_core::Error::message("model is required when provider is set"))?
        .parse()?;

    Ok(Model::remote(provider, id).base_url(string(args, "base_url")?))
}

/// `facebook/bart-large-cnn` and `models/bart` are the same shape, so an explicit scheme or a
/// directory that actually exists is what distinguishes a weight source from a hub id.
fn is_uri(model: &str) -> bool {
    let scheme = model.starts_with("file://") || model.starts_with("http://") || model.starts_with("https://");

    scheme || std::path::Path::new(model).is_dir()
}

fn string(args: &Args, key: &str) -> Result<Option<String>, Error> {
    match args.key(key) {
        v if v.is_undefined() || v.is_none() => Ok(None),
        v => v
            .as_str()
            .map(|value| Some(value.to_string()))
            .ok_or_else(|| nova_core::Error::message(format!("{key} must be a string")).into()),
    }
}
