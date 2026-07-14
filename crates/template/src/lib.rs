mod args;
mod error;
mod minijinja;
mod value;

use std::sync::Arc;

pub use args::*;
pub use error::*;
pub use minijinja::Minijinja;
pub use value::*;

pub trait Context: Send + Sync + std::fmt::Debug {
    fn resolve(&self, name: &str) -> Option<Pointer>;
    fn names(&self) -> Vec<String>;
    fn as_any(&self) -> &dyn std::any::Any;
}

pub trait Engine: Send + Sync {
    fn add_template(&mut self, name: &str, source: &str) -> Result<(), Error>;
    fn render(&self, name: &str, ctx: &Arc<dyn Context>) -> Result<String, Error>;
    fn render_str(&self, source: &str, ctx: &Arc<dyn Context>) -> Result<String, Error>;
    fn eval(&self, expr: &str, ctx: &Arc<dyn Context>) -> Result<Pointer, Error>;
}
