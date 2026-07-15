use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use nova_core::{Args, Binding, Context, Diagnostic, Error, Event, TemplateEngine, event};
use nova_reflect::{Dynamic, Value};

use super::{Arena, Entry, Slot, SlotMut};

#[derive(Clone)]
pub struct Scope(Arc<_Scope>);

struct _Scope {
    trace_id: ulid::Ulid,
    name: String,
    parent: Option<Scope>,
    args: Args,
    engine: Option<Arc<dyn TemplateEngine<Context = Scope>>>,
    symbols: Mutex<HashMap<String, ulid::Ulid>>,
    arena: Arc<Mutex<Arena>>,
    events: crossbeam::Sender<Event>,
}

impl Scope {
    pub(crate) fn new(
        name: impl Into<String>,
        args: impl Into<Args>,
        arena: Arc<Mutex<Arena>>,
        events: crossbeam::Sender<Event>,
    ) -> Self {
        Self(Arc::new(_Scope {
            trace_id: ulid::Ulid::new(),
            name: name.into(),
            args: args.into(),
            parent: None,
            engine: None,
            symbols: Default::default(),
            arena,
            events,
        }))
    }

    pub(crate) fn with_engine(self, engine: Arc<dyn TemplateEngine<Context = Scope>>) -> Self {
        let mut inner = Arc::try_unwrap(self.0).unwrap_or_else(|_| panic!("with_engine on a shared scope"));
        inner.engine = Some(engine);
        Self(Arc::new(inner))
    }

    pub fn trace_id(&self) -> ulid::Ulid {
        self.0.trace_id
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn args(&self) -> &Args {
        &self.0.args
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

    pub fn next(&self, name: impl Into<String>, args: impl Into<Args>) -> Self {
        Self(Arc::new(_Scope {
            trace_id: self.0.trace_id,
            name: name.into(),
            parent: Some(self.clone()),
            args: args.into(),
            engine: self.0.engine.clone(),
            symbols: Default::default(),
            arena: self.0.arena.clone(),
            events: self.0.events.clone(),
        }))
    }

    pub fn iter(&self) -> impl Iterator<Item = (Value, Value)> {
        let mut entries: std::collections::BTreeMap<String, Value> = Default::default();
        self.collect(&mut entries);
        entries.into_iter().map(|(key, value)| (Value::from(key), value))
    }

    fn collect(&self, entries: &mut std::collections::BTreeMap<String, Value>) {
        let keys: Vec<String> = self.0.symbols.lock().unwrap().keys().cloned().collect();

        for key in keys {
            if entries.contains_key(&key) {
                continue;
            }

            if let Some(slot) = Scope::get(self, &key) {
                entries.insert(key, slot.to_dynamic_value());
            }
        }

        if let Some(parent) = &self.0.parent {
            parent.collect(entries);
        }
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

    pub fn set(&self, key: impl AsRef<str>, object: impl Into<Binding>) -> Result<(), Error> {
        let key = key.as_ref();
        let id = self.0.symbols.lock().unwrap().get(key).copied();

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
                self.dispatch(event::object::update(key, from, to.clone()));
            }

            Ok(())
        } else if let Some(parent) = &self.0.parent {
            parent.set(key, object)
        } else {
            Err(Error::message(format!("undeclared variable `{key}`")))
        }
    }

    pub fn declare(&self, key: impl Into<String>, object: impl Into<Binding>) -> &Self {
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

    pub fn call(&self, name: impl AsRef<str>, args: impl Into<Args>) -> Result<Binding, Error> {
        Context::call(self, name.as_ref(), args.into())
    }

    pub fn render(&self, src: &str) -> Result<String, Error> {
        self.try_engine()?.render(src, self)
    }

    pub fn eval(&self, src: &str) -> Result<Value, Error> {
        self.try_engine()?.eval(src, self)
    }

    pub fn dispatch(&self, source: impl Into<event::Source>) -> &Self {
        let _ = self.0.events.send(event::new(self.0.trace_id, self.0.name.clone(), source));
        self
    }

    fn try_engine(&self) -> Result<&Arc<dyn TemplateEngine<Context = Scope>>, Error> {
        self.0
            .engine
            .as_ref()
            .ok_or_else(|| Error::message("no template engine configured"))
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

    fn args(&self) -> &Args {
        &self.0.args
    }

    fn dispatch(&self, source: event::Source) {
        Scope::dispatch(self, source);
    }

    fn next(&self, name: &str, args: Args) -> Arc<dyn Context> {
        Arc::new(Scope::next(self, name, args))
    }

    fn has(&self, key: &str) -> bool {
        Scope::has(self, key)
    }

    fn get(&self, key: &str) -> Option<Binding> {
        if key == "args" {
            return Some(Binding::Value(Value::Dynamic(Dynamic::from_sequence(Arc::new(
                self.0.args.clone(),
            )))));
        }

        if let Some(value) = self.0.args.kargs().get(key) {
            return Some(Binding::Value(value.clone()));
        }

        let slot = Scope::get(self, key)?;
        Some((*slot).clone())
    }

    fn declare(&self, key: &str, value: Value) {
        Scope::declare(self, key, Binding::Value(value));
    }

    fn set(&self, key: &str, value: Value) -> Result<(), Error> {
        Scope::set(self, key, Binding::Value(value))
    }

    fn del(&self, key: &str) {
        Scope::del(self, key);
    }

    fn render(&self, src: &str) -> Result<String, Error> {
        Scope::render(self, src)
    }

    fn eval(&self, src: &str) -> Result<Value, Error> {
        Scope::eval(self, src)
    }
}
