use crate::{Context, Predicate};

pub struct Not(String);

impl Not {
    pub fn new(inner: impl Into<String>) -> Self {
        Self(inner.into())
    }
}

impl Predicate for Not {
    fn invoke(&self, ctx: &Context) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(!ctx.eval(&self.0, ctx.args().clone())?)
    }
}
