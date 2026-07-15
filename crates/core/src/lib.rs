mod args;
mod binding;
mod context;
mod diagnostic;
mod engine;
mod error;
pub mod event;
mod function;
mod manifest;

pub use args::*;
pub use binding::*;
pub use context::*;
pub use diagnostic::*;
pub use engine::*;
pub use error::*;
pub use event::{Event, Observer};
pub use function::*;
pub use manifest::*;

pub trait Action: Send + Sync {
    fn invoke(&self, args: &Args, ctx: &dyn Context) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Predicate: Send + Sync {
    fn invoke(&self, args: &Args, ctx: &dyn Context) -> Result<bool, Box<dyn std::error::Error>>;
}

pub trait Func: Send + Sync {
    fn invoke(&self, args: &Args, ctx: &dyn Context) -> Result<Binding, Box<dyn std::error::Error>>;
}

impl<F> Action for F
where
    F: Fn(&Args, &dyn Context) -> Result<(), Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, args: &Args, ctx: &dyn Context) -> Result<(), Box<dyn std::error::Error>> {
        self(args, ctx)
    }
}

impl<F> Predicate for F
where
    F: Fn(&Args, &dyn Context) -> Result<bool, Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, args: &Args, ctx: &dyn Context) -> Result<bool, Box<dyn std::error::Error>> {
        self(args, ctx)
    }
}

impl<F> Func for F
where
    F: Fn(&Args, &dyn Context) -> Result<Binding, Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, args: &Args, ctx: &dyn Context) -> Result<Binding, Box<dyn std::error::Error>> {
        self(args, ctx)
    }
}
