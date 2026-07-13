use std::path::{Path, PathBuf};
use std::sync::Arc;

use hf_hub::api::sync::{ApiBuilder, ApiRepo};

use super::asset::{Asset, AssetData};
use super::cache;
use super::error::{Error, Result};
use super::model_id::ModelId;
use super::uri::Uri;

const HF_TOKEN: &str = "HF_TOKEN";

pub trait Repository: Send + Sync {
    fn exists(&self, path: &Path) -> bool;

    fn get(&self, path: &Path) -> Result<Asset>;

    fn read(&self, path: &Path) -> Result<AssetData>;

    fn resolve(&self, path: &Path) -> Result<PathBuf>;
}

pub trait DataSource: Send + Sync {
    fn load(&self, key: &str) -> Result<Arc<dyn Repository>>;
}

pub struct HuggingFace {
    repo: Box<ApiRepo>,
}

impl HuggingFace {
    pub fn open(id: &ModelId) -> Result<Self> {
        let mut builder = ApiBuilder::new().with_token(std::env::var(HF_TOKEN).ok());

        if let Some(dir) = cache::dir() {
            builder = builder.with_cache_dir(dir);
        }

        let api = builder.build().map_err(Error::load)?;

        Ok(Self {
            repo: Box::new(api.model(id.to_string())),
        })
    }
}

impl Repository for HuggingFace {
    fn exists(&self, path: &Path) -> bool {
        self.resolve(path).is_ok()
    }

    fn get(&self, path: &Path) -> Result<Asset> {
        Ok(Asset::file(self.resolve(path)?))
    }

    fn read(&self, path: &Path) -> Result<AssetData> {
        let path = self.resolve(path)?;
        Ok(AssetData::File(std::fs::read(path).map_err(Error::load)?))
    }

    fn resolve(&self, path: &Path) -> Result<PathBuf> {
        let file = path.to_str().ok_or_else(|| Error::Load(format!("{path:?} is not utf-8")))?;

        self.repo.get(file).map_err(Error::load)
    }
}

pub struct Directory {
    root: PathBuf,
}

impl Directory {
    pub fn open(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Repository for Directory {
    fn exists(&self, path: &Path) -> bool {
        self.root.join(path).exists()
    }

    fn get(&self, path: &Path) -> Result<Asset> {
        let path = self.resolve(path)?;

        match path.is_dir() {
            true => Ok(Asset::directory(path)),
            false => Ok(Asset::file(path)),
        }
    }

    fn read(&self, path: &Path) -> Result<AssetData> {
        let path = self.resolve(path)?;

        if !path.is_dir() {
            return Ok(AssetData::File(std::fs::read(path).map_err(Error::load)?));
        }

        let mut assets = Vec::new();

        for entry in std::fs::read_dir(path).map_err(Error::load)? {
            let entry = entry.map_err(Error::load)?;
            let path = entry.path();

            assets.push(match path.is_dir() {
                true => Asset::directory(path),
                false => Asset::file(path),
            });
        }

        Ok(AssetData::Directory(assets))
    }

    fn resolve(&self, path: &Path) -> Result<PathBuf> {
        let path = self.root.join(path);

        match path.exists() {
            true => Ok(path),
            false => Err(Error::Load(format!("{} not found", path.display()))),
        }
    }
}

pub struct Http {
    base: Uri,
}

impl Http {
    pub fn open(base: Uri) -> Self {
        Self { base }
    }

    fn cached(&self, file: &str) -> PathBuf {
        cache::dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("urls")
            .join(slug(&self.base.to_string()))
            .join(file)
    }
}

impl Repository for Http {
    fn exists(&self, path: &Path) -> bool {
        self.resolve(path).is_ok()
    }

    fn get(&self, path: &Path) -> Result<Asset> {
        Ok(Asset::file(self.resolve(path)?))
    }

    fn read(&self, path: &Path) -> Result<AssetData> {
        let path = self.resolve(path)?;
        Ok(AssetData::File(std::fs::read(path).map_err(Error::load)?))
    }

    fn resolve(&self, path: &Path) -> Result<PathBuf> {
        let file = path.to_str().ok_or_else(|| Error::Load(format!("{path:?} is not utf-8")))?;
        let cached = self.cached(file);

        if cached.exists() {
            return Ok(cached);
        }

        let url = self.base.join(file)?.to_string();
        let response = reqwest::blocking::get(&url).map_err(Error::network)?;

        if !response.status().is_success() {
            return Err(Error::Network(format!("{url} returned {}", response.status())));
        }

        let bytes = response.bytes().map_err(Error::network)?;

        if let Some(dir) = cached.parent() {
            std::fs::create_dir_all(dir).map_err(Error::load)?;
        }

        std::fs::write(&cached, &bytes).map_err(Error::load)?;

        Ok(cached)
    }
}

fn slug(base: &str) -> String {
    base.chars().map(|ch| if ch.is_alphanumeric() { ch } else { '-' }).collect()
}
