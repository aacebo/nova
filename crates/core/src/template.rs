use nova_reflect::Value;

use crate::{Context, Error};

pub trait TemplateEngine: Send + Sync + 'static {
    type Context: Context;

    fn render(&self, src: &str, ctx: &Self::Context) -> Result<String, Error>;
    fn eval(&self, src: &str, ctx: &Self::Context) -> Result<Value, Error>;
}
