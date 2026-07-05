use std::sync::Arc;

use crate::{Context, Predicate};

pub struct Not(Arc<dyn Predicate>);

impl Not {
    pub fn new(inner: impl Predicate + 'static) -> Self {
        Self(Arc::new(inner))
    }
}

impl Predicate for Not {
    fn invoke(&self, ctx: &Context) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(!self.0.invoke(ctx)?)
    }
}
