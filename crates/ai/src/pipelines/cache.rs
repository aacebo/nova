use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::model::Key;
use crate::resources::Result;

/// A per-pipeline cache of built backends, keyed by transport. Unlike a `OnceLock`, this lets one
/// process hold both a local and a remote backend for the same task.
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
