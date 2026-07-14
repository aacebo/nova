use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{
    Action, Arena, Args, Binding, Diagnostic, Engine, Entry, Event, KArgs, Minijinja, Pointer, Slot, SlotMut, ToType, ToValue,
    Traced, Type, Value, event,
};

#[derive(Clone)]
pub struct Scope(Arc<_Scope>);

struct _Scope {
    trace_id: ulid::Ulid,
    name: String,
    parent: Option<Scope>,
    engine: Arc<dyn Engine>,
    symbols: Mutex<HashMap<String, ulid::Ulid>>,
    arena: Arc<Mutex<Arena>>,
    args: Vec<Pointer>,
    kargs: KArgs,
    events: crossbeam::Sender<Event>,
}

impl Scope {
    pub const KEY: &'static str = "__$scope__";

    pub(crate) fn new(name: impl Into<String>, arena: Arc<Mutex<Arena>>, events: crossbeam::Sender<Event>) -> Self {
        Self(Arc::new(_Scope {
            trace_id: ulid::Ulid::new(),
            name: name.into(),
            parent: None,
            engine: Arc::new(Minijinja::new()),
            symbols: Default::default(),
            arena,
            args: Default::default(),
            kargs: Default::default(),
            events,
        }))
    }

    pub(crate) fn with_engine(self, engine: impl Engine + 'static) -> Self {
        let mut inner = Arc::try_unwrap(self.0).unwrap_or_else(|_| panic!("with_engine on a shared scope"));
        inner.engine = Arc::new(engine);
        Self(Arc::new(inner))
    }

    pub fn trace_id(&self) -> &ulid::Ulid {
        &self.0.trace_id
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn engine(&self) -> &Arc<dyn Engine> {
        &self.0.engine
    }

    pub fn args(&self) -> &[Pointer] {
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
        self.emit(Diagnostic::new(self.0.trace_id).sev(crate::Severity::Error).message(message))
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

    pub fn fork(&self, name: impl Into<String>, args: impl IntoIterator<Item = Pointer>, kargs: impl Into<KArgs>) -> Self {
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

    pub fn get_func(&self, name: &str) -> Result<crate::Function, Box<dyn std::error::Error>> {
        let slot = self
            .get(name)
            .ok_or_else(|| crate::Error::action(self.0.trace_id, name, "not found"))?;

        match slot.as_func() {
            Some(func) => Ok(func.clone()),
            None => Err(Box::new(crate::Error::action(self.0.trace_id, name, "not callable"))),
        }
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

    pub fn call(&self, name: impl AsRef<str>, args: Args) -> Result<Pointer, Box<dyn std::error::Error>> {
        let name = name.as_ref();

        if let Some(slot) = self.get(name)
            && let Some(routine) = slot.as_routine()
        {
            self.dispatch(event::object::call(
                name.to_string(),
                args.args().to_vec(),
                args.kargs().clone().into_inner(),
            ));

            routine.invoke(&args, self)?;
            return Ok(Pointer::new(Value::Null));
        }

        let func = self.get_func(name)?;
        let child = self.fork(name, args.args().to_vec(), args.kargs().clone());

        self.dispatch(event::object::call(
            name.to_string(),
            child.args().to_vec(),
            child.kargs().clone().into_inner(),
        ));

        let child_args = Args::new(child.args().to_vec(), child.kargs().clone());
        func.invoke(&child_args, &child)
    }

    fn as_context(&self) -> Arc<dyn crate::Context> {
        Arc::new(self.clone())
    }

    pub fn eval(&self, src: &str) -> Result<Pointer, Box<dyn std::error::Error>> {
        Ok(self.0.engine.eval(src, &self.as_context())?)
    }

    pub fn render(&self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.0.engine.render(name, &self.as_context())?)
    }

    pub fn render_str(&self, source: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.0.engine.render_str(source, &self.as_context())?)
    }

    pub fn dispatch(&self, source: impl Into<event::Source>) {
        let _ = self.0.events.send(event::new(self.0.trace_id, self.0.name.clone(), source));
    }
}

impl Traced for Scope {
    fn trace_id(&self) -> ulid::Ulid {
        self.0.trace_id
    }
}

impl std::fmt::Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scope")
            .field("trace_id", &self.0.trace_id)
            .finish_non_exhaustive()
    }
}

impl ToType for Scope {
    fn to_type(&self) -> Type {
        Type::Any
    }
}

impl ToValue for Scope {
    fn to_value(&self) -> Value<'_> {
        Value::Undefined
    }
}

impl crate::Context for Scope {
    fn resolve(&self, name: &str) -> Option<Pointer> {
        if let Some(value) = self.kargs().get(name) {
            return Some(value.clone());
        }

        let slot = self.get(name)?;

        match &*slot {
            Binding::Value(value) => Some(value.clone()),
            Binding::Func(func) => Some(Pointer::callable(func.clone())),
            Binding::Routine(rt) => Some(Pointer::callable_namespace(rt.clone())),
        }
    }

    fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.0.symbols.lock().unwrap().keys().cloned().collect();

        if let Some(parent) = &self.0.parent {
            names.extend(crate::Context::names(parent));
        }

        names.sort();
        names.dedup();
        names
    }

    fn as_caller(&self) -> Pointer {
        Pointer::new(self.clone())
    }
}
