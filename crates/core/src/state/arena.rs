use std::collections::HashMap;

use crate::{Entry, Object};

#[derive(Default, Clone)]
pub struct Arena(HashMap<ulid::Ulid, Entry>);

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

    pub fn get(&self, id: &ulid::Ulid) -> Option<Entry> {
        self.0.get(id).cloned()
    }

    pub fn set(&mut self, id: &ulid::Ulid, object: Object) -> &mut Self {
        match self.0.get(id) {
            Some(entry) => {
                *entry.value.write().unwrap() = object;
            }
            None => {
                self.0.insert(*id, Entry::new(object));
            }
        }

        self
    }

    pub fn del(&mut self, id: &ulid::Ulid) -> &mut Self {
        self.0.remove(id);
        self
    }
}
