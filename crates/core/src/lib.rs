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
pub use state::*;

pub type Value = minijinja::Value;
pub type Environment<'a> = minijinja::Environment<'a>;
pub use minijinja::value::Object as Reflect;

pub trait Action: Send + Sync {
    fn invoke(&self, args: &[Value], kargs: &KArgs, scope: &Scope) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Predicate: Send + Sync {
    fn invoke(&self, args: &[Value], kargs: &KArgs, scope: &Scope) -> Result<bool, Box<dyn std::error::Error>>;
}

pub trait Call: Send + Sync {
    fn invoke(&self, args: &[Value], kargs: &KArgs, scope: &Scope) -> Result<Option<Value>, Box<dyn std::error::Error>>;
}

pub trait Import {
    fn import(self, builder: Builder) -> Result<Builder, Box<dyn std::error::Error>>;
}

impl<F> Action for F
where
    F: Fn(&[Value], &KArgs, &Scope) -> Result<(), Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, args: &[Value], kargs: &KArgs, scope: &Scope) -> Result<(), Box<dyn std::error::Error>> {
        self(args, kargs, scope)
    }
}

impl<F> Predicate for F
where
    F: Fn(&[Value], &KArgs, &Scope) -> Result<bool, Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, args: &[Value], kargs: &KArgs, scope: &Scope) -> Result<bool, Box<dyn std::error::Error>> {
        self(args, kargs, scope)
    }
}

impl<F> Call for F
where
    F: Fn(&[Value], &KArgs, &Scope) -> Result<Option<Value>, Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, args: &[Value], kargs: &KArgs, scope: &Scope) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        self(args, kargs, scope)
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

    pub fn call(&self, name: &str, args: impl Into<KArgs>) -> Result<(), Box<dyn std::error::Error>> {
        let args = args.into();
        let scope = self.scope.fork(name, Vec::new(), args.clone());
        scope.call(name, Vec::new(), args)?;
        Ok(())
    }

    pub fn eval(&self, name: &str, args: impl Into<KArgs>) -> Result<bool, Box<dyn std::error::Error>> {
        let args = args.into();
        let scope = self.scope.fork(name, Vec::new(), args.clone());
        Ok(scope.call(name, Vec::new(), args)?.map(|v| v.is_true()).unwrap_or(false))
    }

    pub fn func(&self, name: &str, args: impl Into<KArgs>) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        let args = args.into();
        let scope = self.scope.fork(name, Vec::new(), args.clone());
        scope.call(name, Vec::new(), args)
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

#[doc(hidden)]
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

    pub fn import(mut self, import: impl Import) -> Result<Self, Box<dyn std::error::Error>> {
        self = import.import(self)?;
        Ok(self)
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
                scope.set_local(key.clone(), Object::value(value.clone()));
            }

            root.set(name.clone(), Routine::new(name, scope, manifest.steps));
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
