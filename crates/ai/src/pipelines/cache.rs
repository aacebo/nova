use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::models::ModelRef;
use crate::resources::Result;

/// The api key is fingerprinted rather than stored, so credentials never sit in a long-lived map
/// yet two callers with different keys still get separate clients.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    model: ModelRef,
    credential: u64,
}

impl Key {
    pub fn new(model: &ModelRef, api_key: &Option<String>) -> Self {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        api_key.hash(&mut hasher);

        Self {
            model: model.clone(),
            credential: hasher.finish(),
        }
    }
}

pub struct Cache<T: ?Sized> {
    entries: RwLock<HashMap<Key, Arc<T>>>,
}

impl<T: ?Sized> Cache<T> {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_or_build(&self, key: Key, build: impl FnOnce() -> Result<Arc<T>>) -> Result<Arc<T>> {
        if let Ok(entries) = self.entries.read()
            && let Some(entry) = entries.get(&key)
        {
            return Ok(entry.clone());
        }

        // Built outside the write lock: loading a model is slow, and a duplicate build on a race
        // is cheaper than holding the lock across it.
        let entry = build()?;

        match self.entries.write() {
            Ok(mut entries) => Ok(entries.entry(key).or_insert(entry).clone()),
            Err(_) => Ok(entry),
        }
    }
}

impl<T: ?Sized> Default for Cache<T> {
    fn default() -> Self {
        Self::new()
    }
}
