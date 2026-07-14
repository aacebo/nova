use std::time::Instant;

use crate::{Binding, Entry};

pub struct Slot {
    entry: Entry,
    object: Binding,
}

impl Slot {
    pub(super) fn new(entry: Entry) -> Self {
        let object = entry.value.read().unwrap().clone();
        Self { entry, object }
    }

    pub fn created_at(&self) -> Instant {
        self.entry.created_at
    }
}

impl std::ops::Deref for Slot {
    type Target = Binding;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

pub struct SlotMut {
    entry: Entry,
    object: Binding,
}

impl SlotMut {
    pub(super) fn new(entry: Entry) -> Self {
        let object = entry.value.read().unwrap().clone();
        Self { entry, object }
    }

    pub fn created_at(&self) -> Instant {
        self.entry.created_at
    }
}

impl std::ops::Deref for SlotMut {
    type Target = Binding;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl std::ops::DerefMut for SlotMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}

impl Drop for SlotMut {
    fn drop(&mut self) {
        *self.entry.value.write().unwrap() = self.object.clone();
    }
}
