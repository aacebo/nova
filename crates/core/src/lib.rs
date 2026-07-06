use std::borrow::Cow;

mod arena;
mod args;
pub mod builtin;
mod context;
mod diagnostic;
mod error;
mod object;
mod output;
mod routine;
mod scope;
mod span;

pub use arena::*;
pub use args::*;
pub use context::*;
pub use diagnostic::*;
pub use error::*;
pub use minijinja::context;
pub use object::*;
pub use output::*;
pub use routine::*;
pub use scope::*;
pub use span::*;

pub type Value = minijinja::Value;
pub type Environment<'a> = minijinja::Environment<'a>;

pub trait Action: Send + Sync {
    fn invoke(&self, ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Predicate: Send + Sync {
    fn invoke(&self, ctx: &Context) -> Result<bool, Box<dyn std::error::Error>>;
}

pub trait Map: Send + Sync {
    fn invoke(&self, ctx: &mut Context) -> Result<Option<Value>, Box<dyn std::error::Error>>;
}

pub struct Runtime<'a> {
    env: Environment<'a>,
    scope: Scope,
}

impl<'a> Runtime<'a> {
    pub fn env(&self) -> &Environment<'a> {
        &self.env
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn call(&self, name: &str, args: impl Into<Args>) -> Result<Output, Box<dyn std::error::Error>> {
        let args = args.into();
        let mut ctx = Context::new(ulid::Ulid::new(), args.clone(), &self.env, self.scope.fork());
        ctx.call(name, args)?;
        Ok(ctx.into())
    }

    pub fn eval(&self, name: &str, args: impl Into<Args>) -> Result<Output, Box<dyn std::error::Error>> {
        let args = args.into();
        let ctx = Context::new(ulid::Ulid::new(), args.clone(), &self.env, self.scope.fork());
        let value = ctx.eval(name, args)?;
        let mut output = Output::from(ctx);
        output.value = Some(value.into());
        Ok(output)
    }

    pub fn map(&self, name: &str, args: impl Into<Args>) -> Result<Output, Box<dyn std::error::Error>> {
        let args = args.into();
        let mut ctx = Context::new(ulid::Ulid::new(), args.clone(), &self.env, self.scope.fork());
        let value = ctx.map(name, args)?;
        let mut output = Output::from(ctx);
        output.value = value;
        Ok(output)
    }
}

#[doc(hidden)]
pub struct Builder<'a> {
    env: Environment<'a>,
    scope: Scope,
}

impl<'a> Default for Builder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Builder<'a> {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
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
        self.scope.set(name, action);
        self
    }

    pub fn predicate(self, name: impl Into<String>, predicate: impl Predicate + 'static) -> Self {
        self.scope.set(name, Object::predicate(predicate));
        self
    }

    pub fn map(self, name: impl Into<String>, map: impl Map + 'static) -> Self {
        self.scope.set(name, Object::map(map));
        self
    }

    pub fn routine(self, name: impl Into<String>, entrypoint: impl Into<String>) -> Self {
        self.scope.set(name, Routine::new(entrypoint));
        self
    }

    pub fn global(mut self, name: impl Into<Cow<'a, str>>, value: impl Into<Value>) -> Self {
        self.env.add_global(name, value);
        self
    }

    pub fn template(mut self, name: impl Into<String>, source: impl Into<String>) -> Result<Self, Box<dyn std::error::Error>> {
        self.env.add_template_owned(name.into(), source.into())?;
        Ok(self)
    }

    pub fn filter<N, F, Rv, Args>(mut self, name: N, f: F) -> Self
    where
        N: Into<Cow<'a, str>>,
        F: minijinja::functions::Function<Rv, Args>,
        Rv: minijinja::value::FunctionResult,
        Args: for<'b> minijinja::value::FunctionArgs<'b>,
    {
        self.env.add_filter(name, f);
        self
    }

    pub fn function<N, F, Rv, Args>(mut self, name: N, f: F) -> Self
    where
        N: Into<Cow<'a, str>>,
        F: minijinja::functions::Function<Rv, Args>,
        Rv: minijinja::value::FunctionResult,
        Args: for<'b> minijinja::value::FunctionArgs<'b>,
    {
        self.env.add_function(name, f);
        self
    }

    pub fn build(self) -> Runtime<'a> {
        Runtime {
            env: self.env,
            scope: self.scope,
        }
    }
}
