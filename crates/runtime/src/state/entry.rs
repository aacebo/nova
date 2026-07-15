use std::sync::{Arc, RwLock};
use std::time::Instant;

use nova_core::Binding;

#[derive(Clone)]
pub struct Entry {
    pub value: Arc<RwLock<Binding>>,
    pub created_at: Instant,
}

impl Entry {
    pub(super) fn new(object: Binding) -> Self {
        Self {
            value: Arc::new(RwLock::new(object)),
            created_at: Instant::now(),
        }
    }
}
