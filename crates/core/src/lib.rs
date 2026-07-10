mod args;
mod builtin;
mod diagnostic;
mod error;
pub mod event;
mod manifest;
mod object;
mod state;

pub use args::*;
pub use diagnostic::*;
pub use error::*;
pub use event::{Event, Observer};
pub use manifest::*;
pub use minijinja::context;
pub use object::*;
pub use serde_json::json;
pub use state::*;

pub type Value = minijinja::Value;
pub type Environment<'a> = minijinja::Environment<'a>;
pub use minijinja::value::{Kwargs, Object as Reflect};

pub trait Action: Send + Sync {
    fn invoke(&self, args: &Args, scope: &Scope) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Predicate: Send + Sync {
    fn invoke(&self, args: &Args, scope: &Scope) -> Result<bool, Box<dyn std::error::Error>>;
}

pub trait Call: Send + Sync {
    fn invoke(&self, args: &Args, scope: &Scope) -> Result<Value, Box<dyn std::error::Error>>;
}

impl<F> Action for F
where
    F: Fn(&Args, &Scope) -> Result<(), Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, args: &Args, scope: &Scope) -> Result<(), Box<dyn std::error::Error>> {
        self(args, scope)
    }
}

impl<F> Predicate for F
where
    F: Fn(&Args, &Scope) -> Result<bool, Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, args: &Args, scope: &Scope) -> Result<bool, Box<dyn std::error::Error>> {
        self(args, scope)
    }
}

impl<F> Call for F
where
    F: Fn(&Args, &Scope) -> Result<Value, Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, args: &Args, scope: &Scope) -> Result<Value, Box<dyn std::error::Error>> {
        self(args, scope)
    }
}

pub fn new() -> Builder {
    Builder::new()
}

pub struct Runtime {
    scope: Scope,
    shutdown: Option<crossbeam::Sender<()>>,
    listener: Option<std::thread::JoinHandle<()>>,
}

impl Runtime {
    pub fn env(&self) -> &Environment<'static> {
        self.scope.env()
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn call(&self, name: &str, args: Args) -> Result<(), Box<dyn std::error::Error>> {
        self.scope.call(name, args)?;
        Ok(())
    }

    pub fn eval(&self, name: &str, args: Args) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(self.scope.call(name, args)?.is_true())
    }

    pub fn func(&self, name: &str, args: Args) -> Result<Value, Box<dyn std::error::Error>> {
        self.scope.call(name, args)
    }

    pub fn render(&self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.scope.render(name)
    }

    pub fn render_str(&self, source: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.scope.render_str(source)
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        drop(self.shutdown.take());

        if let Some(handle) = self.listener.take() {
            let _ = handle.join();
        }
    }
}

pub struct Builder {
    scope: Scope,
    templates: Vec<(String, String)>,
    manifests: Vec<Manifest>,
    events: crossbeam::Receiver<Event>,
    observers: Vec<Box<dyn Observer>>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam::unbounded();
        let builder = Self {
            scope: Scope::new("", Default::default(), sender),
            templates: Vec::new(),
            manifests: Vec::new(),
            events: receiver,
            observers: Vec::new(),
        };

        builtin::register(builder)
    }

    pub fn observe(mut self, observer: impl Observer) -> Self {
        self.observers.push(Box::new(observer));
        self
    }

    pub fn var(self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        let name = name.into();
        let value = value.into();
        self.scope.set(name.clone(), Object::value(value));
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
                    existing.merge(manifest);
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
                scope.set_local(key.clone(), Object::value(value.clone()));
            }

            for (key, var) in &manifest.env {
                if let Ok(value) = std::env::var(var) {
                    scope.set_local(key.clone(), Object::value(value.into()));
                }
            }

            let validator = match &manifest.args {
                Some(schema) => {
                    let schema = serde_json::to_value(schema)?;
                    Some(std::sync::Arc::new(jsonschema::validator_for(&schema)?))
                }
                None => None,
            };

            root.set(name.clone(), Routine::new(name, scope, manifest.steps, validator));
        }

        let (shutdown, listener) = if self.observers.is_empty() {
            (None, None)
        } else {
            let events = self.events;
            let observers = self.observers;
            let (shutdown_tx, shutdown_rx) = crossbeam::bounded::<()>(0);
            let handle = std::thread::spawn(move || {
                loop {
                    crossbeam::select! {
                        recv(events) -> event => match event {
                            Ok(event) => {
                                for observer in &observers {
                                    observer.on_event(&event);
                                }
                            }
                            Err(_) => break,
                        },
                        recv(shutdown_rx) -> _ => {
                            for event in events.try_iter() {
                                for observer in &observers {
                                    observer.on_event(&event);
                                }
                            }
                            break;
                        }
                    }
                }
            });

            (Some(shutdown_tx), Some(handle))
        };

        Ok(Runtime {
            scope: root,
            shutdown,
            listener,
        })
    }
}
