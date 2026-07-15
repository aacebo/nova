use std::sync::Arc;

use crate::{Binding, Context, Error};

pub trait Engine: Send + Sync {
    fn add_template(&mut self, name: &str, source: &str) -> Result<(), Error>;
    fn render(&self, name: &str, ctx: &Arc<dyn Context>) -> Result<String, Error>;
    fn render_str(&self, source: &str, ctx: &Arc<dyn Context>) -> Result<String, Error>;
    fn eval(&self, expr: &str, ctx: &Arc<dyn Context>) -> Result<Binding, Error>;
}
