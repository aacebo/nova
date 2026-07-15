use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use nova_core::{Args, Binding, Context, Diagnostic, Engine, Event, KArgs, event};
use nova_reflect::Value;

use super::{Arena, Entry, Slot, SlotMut};

#[derive(Clone)]
pub struct Scope(Arc<_Scope>);

struct _Scope {
    trace_id: ulid::Ulid,
    name: String,
    parent: Option<Scope>,
    engine: Option<Arc<dyn Engine>>,
    symbols: Mutex<HashMap<String, ulid::Ulid>>,
    arena: Arc<Mutex<Arena>>,
    args: Vec<Value>,
    kargs: KArgs,
    events: crossbeam::Sender<Event>,
}

impl Scope {
    pub(crate) fn new(name: impl Into<String>, arena: Arc<Mutex<Arena>>, events: crossbeam::Sender<Event>) -> Self {
        Self(Arc::new(_Scope {
            trace_id: ulid::Ulid::new(),
            name: name.into(),
            parent: None,
            engine: None,
            symbols: Default::default(),
            arena,
            args: Default::default(),
            kargs: Default::default(),
            events,
        }))
    }

    pub(crate) fn with_engine(self, engine: Box<dyn Engine>) -> Self {
        let mut inner = Arc::try_unwrap(self.0).unwrap_or_else(|_| panic!("with_engine on a shared scope"));
        inner.engine = Some(Arc::from(engine));
        Self(Arc::new(inner))
    }

    pub fn trace_id(&self) -> &ulid::Ulid {
        &self.0.trace_id
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn engine(&self) -> Option<&Arc<dyn Engine>> {
        self.0.engine.as_ref()
    }

    pub fn args(&self) -> &[Value] {
        &self.0.args
    }

    pub fn kargs(&self) -> &KArgs {
        &self.0.kargs
    }

    pub fn emit(&self, diagnostic: Diagnostic) -> &Self {
        self.dispatch(diagnostic);
        self
    }

    pub fn error(&self, message: impl Into<String>) -> &Self {
        self.emit(
            Diagnostic::new(self.0.trace_id)
                .sev(nova_core::Severity::Error)
                .message(message),
        )
    }

    pub fn len(&self) -> usize {
        if let Some(parent) = &self.0.parent {
            parent.len() + self.0.symbols.lock().unwrap().len()
        } else {
            self.0.symbols.lock().unwrap().len()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn fork(&self, name: impl Into<String>, args: impl IntoIterator<Item = Value>, kargs: impl Into<KArgs>) -> Self {
        Self(Arc::new(_Scope {
            trace_id: self.0.trace_id,
            name: name.into(),
            parent: Some(self.clone()),
            engine: self.0.engine.clone(),
            symbols: Default::default(),
            arena: self.0.arena.clone(),
            args: args.into_iter().collect(),
            kargs: kargs.into(),
            events: self.0.events.clone(),
        }))
    }

    pub fn has(&self, key: impl AsRef<str>) -> bool {
        if let Some(id) = self.0.symbols.lock().unwrap().get(key.as_ref())
            && self.0.arena.lock().unwrap().has(id)
        {
            true
        } else if let Some(parent) = &self.0.parent {
            parent.has(key)
        } else {
            false
        }
    }

    pub fn entry(&self, key: impl AsRef<str>) -> Option<Entry> {
        let id = self.0.symbols.lock().unwrap().get(key.as_ref()).copied();

        if let Some(id) = id
            && let Some(entry) = self.0.arena.lock().unwrap().get(&id)
        {
            Some(entry)
        } else if let Some(parent) = &self.0.parent {
            parent.entry(key)
        } else {
            None
        }
    }

    pub fn get(&self, key: impl AsRef<str>) -> Option<Slot> {
        self.entry(key).map(Slot::new)
    }

    pub fn get_mut(&self, key: impl AsRef<str>) -> Option<SlotMut> {
        self.entry(key).map(SlotMut::new)
    }

    pub fn set(&self, key: impl Into<String>, object: impl Into<Binding>) -> &Self {
        let key = key.into();
        let id = self.0.symbols.lock().unwrap().get(&key).copied();

        if let Some(id) = id {
            let object = object.into();
            let from = self
                .0
                .arena
                .lock()
                .unwrap()
                .get(&id)
                .and_then(|entry| entry.value.read().unwrap().as_value().cloned());
            self.0.arena.lock().unwrap().set(&id, object.clone());

            if let (Some(from), Some(to)) = (from, object.as_value()) {
                self.dispatch(event::object::update(&key, from, to.clone()));
            }
        } else if let Some(parent) = &self.0.parent {
            parent.set(key, object);
        } else {
            let id = ulid::Ulid::new();
            self.0.symbols.lock().unwrap().insert(key, id);
            self.0.arena.lock().unwrap().set(&id, object.into());
        }

        self
    }

    pub fn set_local(&self, key: impl Into<String>, object: impl Into<Binding>) -> &Self {
        let key = key.into();
        let existing = self.0.symbols.lock().unwrap().get(&key).copied();
        let object = object.into();
        let from = existing.and_then(|id| {
            self.0
                .arena
                .lock()
                .unwrap()
                .get(&id)
                .and_then(|entry| entry.value.read().unwrap().as_value().cloned())
        });

        let id = existing.unwrap_or_else(|| {
            let id = ulid::Ulid::new();
            self.0.symbols.lock().unwrap().insert(key.clone(), id);
            id
        });

        self.0.arena.lock().unwrap().set(&id, object.clone());

        if let (Some(from), Some(to)) = (from, object.as_value()) {
            self.dispatch(event::object::update(&key, from, to.clone()));
        }

        self
    }

    pub fn del(&self, key: &str) -> &Self {
        let id = self.0.symbols.lock().unwrap().remove(key);

        if let Some(id) = id {
            self.0.arena.lock().unwrap().del(&id);
        } else if let Some(parent) = &self.0.parent {
            parent.del(key);
        }

        self
    }

    pub fn call(&self, name: impl AsRef<str>, args: Args) -> Result<Binding, Box<dyn std::error::Error>> {
        let name = name.as_ref();
        let slot = self
            .get(name)
            .ok_or_else(|| nova_core::Error::action(self.0.trace_id, name, "not found"))?;
        let binding = (*slot).clone();
        drop(slot);

        let call = binding
            .as_call()
            .ok_or_else(|| nova_core::Error::action(self.0.trace_id, name, "not callable"))?;

        self.dispatch(event::object::call(
            name.to_string(),
            args.args().to_vec(),
            args.kargs().clone().into_inner(),
        ));

        Ok(call.call(&args, self)?)
    }

    pub fn eval(&self, src: &str) -> Result<Binding, Box<dyn std::error::Error>> {
        Ok(self.try_engine()?.eval(src, &self.as_context())?)
    }

    pub fn render(&self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.try_engine()?.render(name, &self.as_context())?)
    }

    pub fn render_str(&self, source: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.try_engine()?.render_str(source, &self.as_context())?)
    }

    pub fn dispatch(&self, source: impl Into<event::Source>) {
        let _ = self.0.events.send(event::new(self.0.trace_id, self.0.name.clone(), source));
    }

    fn try_engine(&self) -> Result<&Arc<dyn Engine>, nova_core::Error> {
        self.0
            .engine
            .as_ref()
            .ok_or_else(|| nova_core::Error::message("no template engine configured"))
    }

    fn as_context(&self) -> Arc<dyn Context> {
        Arc::new(self.clone())
    }
}

impl std::fmt::Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scope")
            .field("trace_id", &self.0.trace_id)
            .finish_non_exhaustive()
    }
}

impl Context for Scope {
    fn trace_id(&self) -> ulid::Ulid {
        self.0.trace_id
    }

    fn name(&self) -> &str {
        &self.0.name
    }

    fn args(&self) -> &[Value] {
        &self.0.args
    }

    fn kargs(&self) -> &KArgs {
        &self.0.kargs
    }

    fn resolve(&self, name: &str) -> Option<Binding> {
        if name == "args" {
            return Some(Binding::from(self.args().to_vec()));
        }

        if let Some(value) = self.kargs().get(name) {
            return Some(Binding::Value(value.clone()));
        }

        let slot = self.get(name)?;
        Some((*slot).clone())
    }

    fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.0.symbols.lock().unwrap().keys().cloned().collect();

        if let Some(parent) = &self.0.parent {
            names.extend(Context::names(parent));
        }

        names.sort();
        names.dedup();
        names
    }

    fn call(&self, name: &str, args: Args) -> Result<Binding, nova_core::Error> {
        Scope::call(self, name, args).map_err(into_core_error)
    }

    fn eval(&self, expr: &str) -> Result<Binding, nova_core::Error> {
        Scope::eval(self, expr).map_err(into_core_error)
    }

    fn render(&self, name: &str) -> Result<String, nova_core::Error> {
        Scope::render(self, name).map_err(into_core_error)
    }

    fn render_str(&self, source: &str) -> Result<String, nova_core::Error> {
        Scope::render_str(self, source).map_err(into_core_error)
    }

    fn dispatch(&self, source: event::Source) {
        Scope::dispatch(self, source);
    }

    fn fork(&self, name: &str, args: Vec<Value>, kargs: KArgs) -> Arc<dyn Context> {
        Arc::new(Scope::fork(self, name, args, kargs))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn into_core_error(err: Box<dyn std::error::Error>) -> nova_core::Error {
    match err.downcast::<nova_core::Error>() {
        Ok(err) => *err,
        Err(err) => nova_core::Error::message(err.to_string()),
    }
}
