use crate::{Args, Predicate, call};

pub struct Not(String);

impl Not {
    pub fn new(inner: impl Into<String>) -> Self {
        Self(inner.into())
    }
}

impl Predicate for Not {
    fn invoke(&self, args: &Args) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(!call!(&self.0, args.clone()).map(|v| v.is_true()).unwrap_or(false))
    }
}
