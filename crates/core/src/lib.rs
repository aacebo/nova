mod args;
mod binding;
mod context;
mod diagnostic;
mod error;
pub mod event;
mod function;
mod manifest;
mod template;

pub use args::*;
pub use binding::*;
pub use context::*;
pub use diagnostic::*;
pub use error::*;
pub use event::{Event, Observer};
pub use function::*;
pub use manifest::*;
pub use template::*;

pub trait Action: Send + Sync {
    fn invoke(&self, ctx: &dyn Context) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Predicate: Send + Sync {
    fn invoke(&self, ctx: &dyn Context) -> Result<bool, Box<dyn std::error::Error>>;
}

pub trait Func: Send + Sync {
    fn invoke(&self, ctx: &dyn Context) -> Result<Binding, Box<dyn std::error::Error>>;
}

impl<F> Action for F
where
    F: Fn(&dyn Context) -> Result<(), Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, ctx: &dyn Context) -> Result<(), Box<dyn std::error::Error>> {
        self(ctx)
    }
}

impl<F> Predicate for F
where
    F: Fn(&dyn Context) -> Result<bool, Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, ctx: &dyn Context) -> Result<bool, Box<dyn std::error::Error>> {
        self(ctx)
    }
}

impl<F> Func for F
where
    F: Fn(&dyn Context) -> Result<Binding, Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, ctx: &dyn Context) -> Result<Binding, Box<dyn std::error::Error>> {
        self(ctx)
    }
}
