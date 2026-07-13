use std::sync::Arc;

use candle_core::{DType, Device};

use crate::resources::{Directory, Error, Http, HuggingFace, Loader, ModelId, Provider, Repository, Resource, Result, Uri};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Model {
    Hub(ModelId),
    Local(Resource),
    Remote {
        provider: Provider,
        id: ModelId,
        base_url: Option<String>,
    },
}

impl Model {
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

    pub fn repository(&self) -> Result<Arc<dyn Repository>> {
        match self {
            Self::Hub(id) => Ok(Arc::new(HuggingFace::open(id)?)),
            Self::Local(resource) => match &resource.uri {
                Uri::Local(path) => Ok(Arc::new(Directory::open(path))),
                Uri::Http(_) => Ok(Arc::new(Http::open(resource.uri.clone()))),
            },
            Self::Remote { id, .. } => Err(Error::Load(format!("{id} is a remote model and has no weights"))),
        }
    }

    pub fn loader(&self, device: Device, dtype: DType) -> Result<Loader> {
        Ok(Loader::new(self.repository()?, device, dtype))
    }
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hub(id) => write!(f, "{id}"),
            Self::Local(resource) => write!(f, "{resource}"),
            Self::Remote { provider, id, .. } => write!(f, "{provider}:{id}"),
        }
    }
}

/// A cache key. The api key is never stored — only a fingerprint of it — so credentials do not sit
/// in a long-lived map, yet two callers with different keys still get separate clients.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    model: Model,
    credential: u64,
}

impl Key {
    pub fn new(model: &Model, api_key: &Option<String>) -> Self {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        api_key.hash(&mut hasher);

        Self {
            model: model.clone(),
            credential: hasher.finish(),
        }
    }
}
