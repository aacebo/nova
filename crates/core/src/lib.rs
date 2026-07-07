use std::sync::Arc;

mod args;
pub mod builtin;
mod context;
mod diagnostic;
mod error;
mod object;
mod output;
mod routine;
mod span;
mod state;

pub use args::*;
pub use context::*;
pub use diagnostic::*;
pub use error::*;
pub use minijinja::context;
pub use object::*;
pub use output::*;
pub use routine::*;
pub use span::*;
pub use state::*;

pub type Value = minijinja::Value;
pub type Environment<'a> = minijinja::Environment<'a>;

pub trait Action: Send + Sync {
    fn invoke(&self, ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Predicate: Send + Sync {
    fn invoke(&self, ctx: &Context) -> Result<bool, Box<dyn std::error::Error>>;
}

pub trait Call: Send + Sync {
    fn invoke(&self, ctx: &mut Context) -> Result<Option<Value>, Box<dyn std::error::Error>>;
}

impl<F> Action for F
where
    F: Fn(&mut Context) -> Result<(), Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
        self(ctx)
    }
}

impl<F> Predicate for F
where
    F: Fn(&Context) -> Result<bool, Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, ctx: &Context) -> Result<bool, Box<dyn std::error::Error>> {
        self(ctx)
    }
}

impl<F> Call for F
where
    F: Fn(&mut Context) -> Result<Option<Value>, Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, ctx: &mut Context) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        self(ctx)
    }
}

pub struct Runtime {
    env: Arc<Environment<'static>>,
    scope: Scope,
}

impl Runtime {
    pub fn env(&self) -> &Environment<'static> {
        &self.env
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn call(&self, name: &str, args: impl Into<Args>) -> Result<Output, Box<dyn std::error::Error>> {
        let args = args.into();
        let ctx = Context::new(self.env.clone(), self.scope.fork(args.clone()));
        ctx.call(name, args)?;
        Ok(ctx.into())
    }

    pub fn eval(&self, name: &str, args: impl Into<Args>) -> Result<Output, Box<dyn std::error::Error>> {
        let args = args.into();
        let ctx = Context::new(self.env.clone(), self.scope.fork(args.clone()));
        let value = ctx.call(name, args)?.map(|v| v.is_true()).unwrap_or(false);
        let mut output = Output::from(ctx);
        output.value = Some(value.into());
        Ok(output)
    }

    pub fn func(&self, name: &str, args: impl Into<Args>) -> Result<Output, Box<dyn std::error::Error>> {
        let args = args.into();
        let ctx = Context::new(self.env.clone(), self.scope.fork(args.clone()));
        let value = ctx.call(name, args)?;
        let mut output = Output::from(ctx);
        output.value = value;
        Ok(output)
    }
}

#[doc(hidden)]
pub struct Builder {
    templates: Vec<(String, String)>,
    scope: Scope,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
            scope: Scope::from(Arena::new()),
        }
    }

    pub fn var(self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        let name = name.into();
        let value = value.into();
        self.scope.set(name.clone(), Var::new(name, value));
        self
    }

    pub fn action(self, name: impl Into<String>, action: impl Action + 'static) -> Self {
        let name = name.into();
        self.scope.set(name.clone(), Object::action(name, action));
        self
    }

    pub fn predicate(self, name: impl Into<String>, predicate: impl Predicate + 'static) -> Self {
        let name = name.into();
        self.scope.set(name.clone(), Object::predicate(name, predicate));
        self
    }

    pub fn func(self, name: impl Into<String>, func: impl Call + 'static) -> Self {
        let name = name.into();
        self.scope.set(name.clone(), Object::func(name, func));
        self
    }

    pub fn routine(self, name: impl Into<String>, entrypoint: impl Into<String>) -> Self {
        let name = name.into();
        self.scope.set(name.clone(), Object::action(name, Routine::new(entrypoint)));
        self
    }

    pub fn template(mut self, name: impl Into<String>, source: impl Into<String>) -> Self {
        self.templates.push((name.into(), source.into()));
        self
    }

    pub fn build(self) -> Result<Runtime, Box<dyn std::error::Error>> {
        let mut env = Environment::new();

        for (name, source) in self.templates {
            env.add_template_owned(name, source)?;
        }

        Ok(Runtime {
            env: Arc::new(env),
            scope: self.scope,
        })
    }
}
