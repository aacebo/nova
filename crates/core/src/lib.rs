mod bind;
mod builtin;
mod diagnostic;
mod error;
pub mod event;
mod manifest;
mod state;

pub use bind::*;
pub use diagnostic::*;
pub use error::*;
pub use event::{Event, Observer};
pub use manifest::*;
use nova_reflect::Value;
use nova_template::{Args, Engine, KArgs, Minijinja, Pointer, is_truthy};
pub use state::*;

pub trait Action: Send + Sync {
    fn invoke(&self, args: &Args, scope: &Scope) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Predicate: Send + Sync {
    fn invoke(&self, args: &Args, scope: &Scope) -> Result<bool, Box<dyn std::error::Error>>;
}

pub trait Call: Send + Sync {
    fn invoke(&self, args: &Args, scope: &Scope) -> Result<Pointer, Box<dyn std::error::Error>>;
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
    F: Fn(&Args, &Scope) -> Result<Pointer, Box<dyn std::error::Error>> + Send + Sync,
{
    fn invoke(&self, args: &Args, scope: &Scope) -> Result<Pointer, Box<dyn std::error::Error>> {
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
    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn call(&self, name: &str, args: Args) -> Result<(), Box<dyn std::error::Error>> {
        self.scope.call(name, args)?;
        Ok(())
    }

    pub fn eval(&self, name: &str, args: Args) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(is_truthy(&self.scope.call(name, args)?.value()))
    }

    pub fn func(&self, name: &str, args: Args) -> Result<Pointer, Box<dyn std::error::Error>> {
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

type EngineFactory = Box<dyn Fn() -> Box<dyn Engine> + Send + Sync>;

pub struct Builder {
    scope: Scope,
    templates: Vec<(String, String)>,
    manifests: Vec<Manifest>,
    events: crossbeam::Receiver<Event>,
    observers: Vec<Box<dyn Observer>>,
    engine: EngineFactory,
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
            engine: Box::new(|| Box::new(Minijinja::new())),
        };

        builtin::register(builder)
    }

    pub fn observe(mut self, observer: impl Observer) -> Self {
        self.observers.push(Box::new(observer));
        self
    }

    pub fn var(self, name: impl Into<String>, value: impl Into<Pointer>) -> Self {
        let name = name.into();
        let value = value.into();
        self.scope.set(name.clone(), Binding::value(value));
        self
    }

    pub fn vars(mut self, values: impl IntoIterator<Item = (impl Into<String>, impl Into<Pointer>)>) -> Self {
        for (name, value) in values {
            self = self.var(name, value);
        }

        self
    }

    pub fn action(self, name: impl Into<String>, action: impl Action + 'static) -> Self {
        let name = name.into();
        self.scope.set(name.clone(), Binding::action(name, action));
        self
    }

    pub fn predicate(self, name: impl Into<String>, predicate: impl Predicate + 'static) -> Self {
        let name = name.into();
        self.scope.set(name.clone(), Binding::predicate(name, predicate));
        self
    }

    pub fn func(self, name: impl Into<String>, func: impl Call + 'static) -> Self {
        let name = name.into();
        self.scope.set(name.clone(), Binding::func(name, func));
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
        E: Engine + 'static,
        F: Fn() -> E + Send + Sync + 'static,
    {
        self.engine = Box::new(move || Box::new(factory()));
        self
    }

    pub fn build(self) -> Result<Runtime, Box<dyn std::error::Error>> {
        let mut env = (self.engine)();

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
            let mut cenv = (self.engine)();

            for (tmpl, source) in &manifest.templates {
                cenv.add_template(tmpl, source)?;
            }

            let scope = root.fork(&name, Vec::new(), KArgs::new()).with_engine(cenv);

            for (key, value) in &manifest.vars {
                scope.set_local(key.clone(), Binding::value(value.clone()));
            }

            for (key, var) in &manifest.env {
                if let Ok(value) = std::env::var(var) {
                    scope.set_local(key.clone(), Binding::value(Pointer::new(Value::from(value))));
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
