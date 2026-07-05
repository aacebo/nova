use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::Object;

#[derive(Default, Clone)]
pub struct Arena(HashMap<ulid::Ulid, Arc<RwLock<Object>>>);

impl Arena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn has(&self, id: &ulid::Ulid) -> bool {
        self.0.contains_key(id)
    }

    pub fn get(&self, id: &ulid::Ulid) -> Option<Arc<RwLock<Object>>> {
        self.0.get(id).cloned()
    }

    pub fn set(&mut self, id: &ulid::Ulid, object: Object) -> &mut Self {
        self.0.insert(*id, Arc::new(RwLock::new(object)));
        self
    }

    pub fn del(&mut self, id: &ulid::Ulid) -> &mut Self {
        self.0.remove(id);
        self
    }
}
