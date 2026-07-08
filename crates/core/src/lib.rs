mod args;
mod builtin;
mod diagnostic;
mod error;
mod global;
mod manifest;
mod object;
mod output;
mod span;
mod state;

pub use args::*;
pub use builtin::*;
pub use diagnostic::*;
pub use error::*;
pub use global::*;
pub use manifest::*;
pub use minijinja::context;
pub use object::*;
pub use output::*;
pub use span::*;
pub use state::*;

pub type Value = minijinja::Value;
pub type Environment<'a> = minijinja::Environment<'a>;

pub trait Action: Send + Sync {
    fn invoke(&self, args: &[Value], kargs: &KArgs) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Predicate: Send + Sync {
    fn invoke(&self, args: &[Value], kargs: &KArgs) -> Result<bool, Box<dyn std::error::Error>>;
}

pub trait Call: Send + Sync {
    fn invoke(&self, args: &[Value], kargs: &KArgs) -> Result<Option<Value>, Box<dyn std::error::Error>>;
}

pub trait Observer: Send + Sync {
    #![allow(unused)]

    fn on_create(&self, object: &Object) {}
    fn on_update(&self, object: &Object) {}
    fn on_delete(&self, object: &Object) {}

    fn on_before_call(&self, object: &Object) {}
    fn on_after_call(&self, object: &Object) {}
}

impl<F> Action for F
where
    F: Fn(&[Value], &KArgs) -> Result<(), Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, args: &[Value], kargs: &KArgs) -> Result<(), Box<dyn std::error::Error>> {
        self(args, kargs)
    }
}

impl<F> Predicate for F
where
    F: Fn(&[Value], &KArgs) -> Result<bool, Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, args: &[Value], kargs: &KArgs) -> Result<bool, Box<dyn std::error::Error>> {
        self(args, kargs)
    }
}

impl<F> Call for F
where
    F: Fn(&[Value], &KArgs) -> Result<Option<Value>, Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, args: &[Value], kargs: &KArgs) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        self(args, kargs)
    }
}

pub fn new() -> Builder {
    Builder::new()
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

    pub fn call(&self, name: &str, args: impl Into<KArgs>) -> Result<Output, Box<dyn std::error::Error>> {
        let args = args.into();
        let scope = self.scope.fork(name, Vec::new(), args.clone());
        scope.call(name, Vec::new(), args)?;
        Ok(scope.into())
    }

    pub fn eval(&self, name: &str, args: impl Into<KArgs>) -> Result<Output, Box<dyn std::error::Error>> {
        let args = args.into();
        let scope = self.scope.fork(name, Vec::new(), args.clone());
        let value = scope.call(name, Vec::new(), args)?.map(|v| v.is_true()).unwrap_or(false);
        let mut output = Output::from(scope);
        output.value = Some(value.into());
        Ok(output)
    }

    pub fn func(&self, name: &str, args: impl Into<KArgs>) -> Result<Output, Box<dyn std::error::Error>> {
        let args = args.into();
        let scope = self.scope.fork(name, Vec::new(), args.clone());
        let value = scope.call(name, Vec::new(), args)?;
        let mut output = Output::from(scope);
        output.value = value;
        Ok(output)
    }
}

#[doc(hidden)]
pub struct Builder {
    scope: Scope,
    templates: Vec<(String, String)>,
    manifests: Vec<Manifest>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        let builder = Self {
            scope: Scope::new("", Default::default()),
            templates: Vec::new(),
            manifests: Vec::new(),
        };

        builtin::register(builder)
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

    pub fn routine(mut self, manifest: impl Into<Manifest>) -> Self {
        self.manifests.push(manifest.into());
        self
    }

    pub fn build(self) -> Result<Runtime, Box<dyn std::error::Error>> {
        let mut env = Environment::new();

        for (name, source) in self.templates {
            env.add_template_owned(name, source)?;
        }

        let root = self.scope.with_env(env);
        let mut merged: std::collections::BTreeMap<String, Manifest> = std::collections::BTreeMap::new();

        for manifest in self.manifests {
            match merged.get_mut(&manifest.name) {
                Some(existing) => {
                    existing.on.extend(manifest.on);
                    existing.vars.extend(manifest.vars);
                    existing.templates.extend(manifest.templates);
                    existing.steps.extend(manifest.steps);
                }
                None => {
                    merged.insert(manifest.name.clone(), manifest);
                }
            }
        }

        for (name, manifest) in merged {
            let mut cenv = Environment::new();

            for (tmpl, source) in &manifest.templates {
                cenv.add_template_owned(tmpl.clone(), source.clone())?;
            }

            let scope = root.fork(&name, Vec::new(), KArgs::new()).with_env(cenv);

            for (key, value) in &manifest.vars {
                scope.set_local(key.clone(), Var::new(key.clone(), value.clone()));
            }

            root.set(name.clone(), Routine::new(name, scope, manifest.steps));
        }

        Ok(Runtime { scope: root })
    }
}
