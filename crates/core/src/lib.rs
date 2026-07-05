use std::borrow::Cow;
use std::sync::Arc;

mod arena;
mod args;
pub mod builtin;
mod context;
mod diagnostic;
mod error;
mod object;
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

    /// Top-level entry point: invoke a registered action by name.
    ///
    /// Resolves `name` against the runtime's scope, forks a fresh scope for the call,
    /// and mints a new `trace_id` for the resulting invocation.
    pub fn invoke(&self, name: &str, args: impl Into<Args>) -> Result<(), Box<dyn std::error::Error>> {
        let trace_id = ulid::Ulid::new();
        let object = self
            .scope
            .get(name)
            .ok_or_else(|| Error::action(trace_id, name, "action not found"))?;

        let action = {
            let guard = object.read().map_err(|_| Error::message("scope lock poisoned"))?;
            match &*guard {
                Object::Action(action) => Arc::clone(action),
                _ => return Err(Box::new(Error::action(trace_id, name, "action not found"))),
            }
        };

        let mut ctx = Context::new(trace_id, args.into(), &self.env, self.scope.fork());
        action.invoke(&mut ctx)
    }

    /// Top-level entry point: evaluate a registered predicate by name.
    ///
    /// Resolves `name` against the runtime's scope, forks a fresh scope for the call,
    /// and mints a new `trace_id` for the resulting evaluation.
    pub fn eval(&self, name: &str, args: impl Into<Args>) -> Result<bool, Box<dyn std::error::Error>> {
        let trace_id = ulid::Ulid::new();
        let object = self
            .scope
            .get(name)
            .ok_or_else(|| Error::action(trace_id, name, "predicate not found"))?;

        let predicate = {
            let guard = object.read().map_err(|_| Error::message("scope lock poisoned"))?;
            match &*guard {
                Object::Predicate(predicate) => Arc::clone(predicate),
                _ => return Err(Box::new(Error::action(trace_id, name, "predicate not found"))),
            }
        };

        let ctx = Context::new(trace_id, args.into(), &self.env, self.scope.fork());
        predicate.invoke(&ctx)
    }

    /// Top-level entry point: invoke a registered map by name and return its value.
    ///
    /// Resolves `name` against the runtime's scope, forks a fresh scope for the call,
    /// and mints a new `trace_id` for the resulting invocation.
    pub fn map(&self, name: &str, args: impl Into<Args>) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        let trace_id = ulid::Ulid::new();
        let object = self
            .scope
            .get(name)
            .ok_or_else(|| Error::action(trace_id, name, "map not found"))?;

        let map = {
            let guard = object.read().map_err(|_| Error::message("scope lock poisoned"))?;
            match &*guard {
                Object::Map(map) => Arc::clone(map),
                _ => return Err(Box::new(Error::action(trace_id, name, "map not found"))),
            }
        };

        let mut ctx = Context::new(trace_id, args.into(), &self.env, self.scope.fork());
        map.invoke(&mut ctx)
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
