mod builtin;
pub mod engine;
mod routine;
mod state;
mod step;

use nova_core::{Action, Args, Binding, Event, Function, Manifest, Observer, Predicate};
use nova_reflect::Value;
pub use routine::*;
pub use state::*;

pub fn new() -> Builder {
    Builder::new()
}

pub struct Runtime {
    scope: Scope,
    shutdown: Option<crossbeam::Sender<()>>,
    listener: Option<std::thread::JoinHandle<()>>,
}

impl Runtime {
    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn call(&self, name: &str, args: Args) -> Result<(), Box<dyn std::error::Error>> {
        self.scope.call(name, args)?;
        Ok(())
    }

    pub fn eval(&self, name: &str, args: Args) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(self.scope.call(name, args)?.is_truthy())
    }

    pub fn func(&self, name: &str, args: Args) -> Result<Binding, Box<dyn std::error::Error>> {
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

impl TryFrom<Manifest> for Runtime {
    type Error = Box<dyn std::error::Error>;

    fn try_from(manifest: Manifest) -> Result<Self, Self::Error> {
        new().routine(manifest).build()
    }
}

impl TryFrom<Vec<Manifest>> for Runtime {
    type Error = Box<dyn std::error::Error>;

    fn try_from(manifests: Vec<Manifest>) -> Result<Self, Self::Error> {
        let mut builder = new();

        for manifest in manifests {
            builder = builder.routine(manifest);
        }

        builder.build()
    }
}

type EngineFactory = Box<dyn Fn() -> Box<dyn nova_core::Engine> + Send + Sync>;

pub struct Builder {
    scope: Scope,
    templates: Vec<(String, String)>,
    manifests: Vec<Manifest>,
    events: crossbeam::Receiver<Event>,
    observers: Vec<Box<dyn Observer>>,
    engine: Option<EngineFactory>,
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
            engine: default_engine(),
        };

        builtin::register(builder)
    }

    pub fn observe(mut self, observer: impl Observer) -> Self {
        self.observers.push(Box::new(observer));
        self
    }

    pub fn var(self, name: impl Into<String>, value: impl Into<Binding>) -> Self {
        let name = name.into();
        let value = value.into();
        self.scope.set(name.clone(), value);
        self
    }

    pub fn vars(mut self, values: impl IntoIterator<Item = (impl Into<String>, impl Into<Binding>)>) -> Self {
        for (name, value) in values {
            self = self.var(name, value);
        }

        self
    }

    pub fn action(self, name: impl Into<String>, action: impl Action + 'static) -> Self {
        let name = name.into();
        self.scope.set(name.clone(), Function::action(name, action));
        self
    }

    pub fn predicate(self, name: impl Into<String>, predicate: impl Predicate + 'static) -> Self {
        let name = name.into();
        self.scope.set(name.clone(), Function::predicate(name, predicate));
        self
    }

    pub fn func(self, name: impl Into<String>, func: impl nova_core::Func + 'static) -> Self {
        let name = name.into();
        self.scope.set(name.clone(), Function::func(name, func));
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

    pub fn engine<E, F>(mut self, factory: F) -> Self
    where
        E: nova_core::Engine + 'static,
        F: Fn() -> E + Send + Sync + 'static,
    {
        self.engine = Some(Box::new(move || Box::new(factory())));
        self
    }

    pub fn build(self) -> Result<Runtime, Box<dyn std::error::Error>> {
        let factory = self
            .engine
            .ok_or_else(|| nova_core::Error::message("no template engine configured"))?;
        let mut env = factory();

        for (name, source) in self.templates {
            env.add_template(&name, &source)?;
        }

        let root = self.scope.with_engine(env);
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
            let mut cenv = factory();

            for (tmpl, source) in &manifest.templates {
                cenv.add_template(tmpl, source)?;
            }

            let scope = root.fork(&name, Vec::new(), nova_core::KArgs::new()).with_engine(cenv);

            for (key, value) in &manifest.vars {
                scope.set_local(key.clone(), value.clone());
            }

            for (key, var) in &manifest.env {
                if let Ok(value) = std::env::var(var) {
                    scope.set_local(key.clone(), Binding::new(Value::from(value)));
                }
            }

            let validator = manifest.args.clone().map(std::sync::Arc::new);
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

#[cfg(feature = "minijinja")]
fn default_engine() -> Option<EngineFactory> {
    Some(Box::new(|| Box::new(engine::Minijinja::new())))
}

#[cfg(not(feature = "minijinja"))]
fn default_engine() -> Option<EngineFactory> {
    None
}
