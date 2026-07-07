use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::Object;

#[derive(Clone)]
pub struct Entry {
    pub value: Arc<RwLock<Object>>,
    pub created_at: Instant,
}

impl Entry {
    pub(super) fn new(object: Object) -> Self {
        Self {
            value: Arc::new(RwLock::new(object)),
            created_at: Instant::now(),
        }
    }
}
