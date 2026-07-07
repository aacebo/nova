mod args;
pub mod builtin;
mod diagnostic;
mod error;
mod global;
mod manifest;
mod object;
mod output;
mod routine;
mod span;
mod state;

pub use args::*;
pub use diagnostic::*;
pub use error::*;
pub use global::*;
pub use manifest::*;
pub use minijinja::context;
pub use object::*;
pub use output::*;
pub use routine::*;
pub use span::*;
pub use state::*;

pub type Value = minijinja::Value;
pub type Environment<'a> = minijinja::Environment<'a>;

pub trait Action: Send + Sync {
    fn invoke(&self, args: &Args) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Predicate: Send + Sync {
    fn invoke(&self, args: &Args) -> Result<bool, Box<dyn std::error::Error>>;
}

pub trait Call: Send + Sync {
    fn invoke(&self, args: &Args) -> Result<Option<Value>, Box<dyn std::error::Error>>;
}

impl<F> Action for F
where
    F: Fn(&Args) -> Result<(), Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
        self(args)
    }
}

impl<F> Predicate for F
where
    F: Fn(&Args) -> Result<bool, Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, args: &Args) -> Result<bool, Box<dyn std::error::Error>> {
        self(args)
    }
}

impl<F> Call for F
where
    F: Fn(&Args) -> Result<Option<Value>, Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, args: &Args) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        self(args)
    }
}

pub fn new() -> Builder {
    Builder::new()
}

pub fn load(manifest: Manifest) -> Builder {
    let entrypoint = manifest.name.clone().unwrap_or_else(|| "main".into());

    new()
        .vars(manifest.vars)
        .templates(manifest.templates)
        .steps(entrypoint, manifest.steps)
}

pub struct Runtime {
    scope: Scope,
}

impl Runtime {
    pub fn env(&self) -> &Environment<'static> {
        self.scope.env()
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn call(&self, name: &str, args: impl Into<Args>) -> Result<Output, Box<dyn std::error::Error>> {
        let args = args.into();
        let scope = self.scope.fork(args.clone());
        scope.call(name, args)?;
        Ok(scope.into())
    }

    pub fn eval(&self, name: &str, args: impl Into<Args>) -> Result<Output, Box<dyn std::error::Error>> {
        let args = args.into();
        let scope = self.scope.fork(args.clone());
        let value = scope.call(name, args)?.map(|v| v.is_true()).unwrap_or(false);
        let mut output = Output::from(scope);
        output.value = Some(value.into());
        Ok(output)
    }

    pub fn func(&self, name: &str, args: impl Into<Args>) -> Result<Output, Box<dyn std::error::Error>> {
        let args = args.into();
        let scope = self.scope.fork(args.clone());
        let value = scope.call(name, args)?;
        let mut output = Output::from(scope);
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
        builtin::register(Self {
            templates: Vec::new(),
            scope: Scope::from(Arena::new()),
        })
    }

    pub fn var(self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        let name = name.into();
        let value = value.into();
        self.scope.set(name.clone(), Var::new(name, value));
        self
    }

    pub fn vars(mut self, values: impl IntoIterator<Item = (impl Into<String>, impl Into<Value>)>) -> Self {
        for (name, value) in values {
            self = self.var(name, value);
        }
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

    pub fn templates(mut self, values: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
        for (name, source) in values {
            self = self.template(name, source);
        }
        self
    }

    pub fn step(self, name: impl Into<String>, step: impl Into<Step>) -> Self {
        self.action(name, step.into())
    }

    pub fn steps(self, entrypoint: impl Into<String>, steps: impl IntoIterator<Item = impl Into<Step>>) -> Self {
        let entrypoint = entrypoint.into();
        let mut names = Vec::new();
        let mut this = self;

        for (index, step) in steps.into_iter().enumerate() {
            let name = format!("{}[{}]", entrypoint, index);
            this = this.step(name.clone(), step);
            names.push(name);
        }

        this.action(entrypoint, Sequence::new(names))
    }

    pub fn build(self) -> Result<Runtime, Box<dyn std::error::Error>> {
        let mut env = Environment::new();

        for (name, source) in self.templates {
            env.add_template_owned(name, source)?;
        }

        Ok(Runtime {
            scope: self.scope.with_env(env),
        })
    }
}
