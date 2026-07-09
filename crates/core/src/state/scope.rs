use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{
    Action, Arena, Args, Diagnostic, Entry, Environment, Event, KArgs, Object, Reflect, Slot, SlotMut, Traced, Value, event,
};

#[derive(Clone)]
pub struct Scope(Arc<_Scope>);

struct _Scope {
    trace_id: ulid::Ulid,
    name: String,
    parent: Option<Scope>,
    env: Arc<Environment<'static>>,
    symbols: Mutex<HashMap<String, ulid::Ulid>>,
    arena: Arc<Mutex<Arena>>,
    args: Vec<Value>,
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
            env: Default::default(),
            symbols: Default::default(),
            arena,
            args: Default::default(),
            kargs: Default::default(),
            events,
        }))
    }

    pub(crate) fn with_env(self, env: Environment<'static>) -> Self {
        let mut inner = Arc::try_unwrap(self.0).unwrap_or_else(|_| panic!("with_env on a shared scope"));
        inner.env = Arc::new(env);
        Self(Arc::new(inner))
    }

    pub fn trace_id(&self) -> &ulid::Ulid {
        &self.0.trace_id
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn env(&self) -> &Environment<'static> {
        &self.0.env
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

    pub fn fork(&self, name: impl Into<String>, args: impl IntoIterator<Item = Value>, kargs: impl Into<KArgs>) -> Self {
        Self(Arc::new(_Scope {
            trace_id: self.0.trace_id,
            name: name.into(),
            parent: Some(self.clone()),
            env: self.0.env.clone(),
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

    pub fn set(&self, key: impl Into<String>, object: impl Into<Object>) -> &Self {
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

    pub fn set_local(&self, key: impl Into<String>, object: impl Into<Object>) -> &Self {
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

    pub fn call(&self, name: impl AsRef<str>, args: Args) -> Result<Value, Box<dyn std::error::Error>> {
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
            return Ok(Value::from(()));
        }

        let func = self.get_func(name)?;
        let child = self.fork(name, args.args().to_vec(), args.kargs().clone());

        self.dispatch(event::object::call(
            name.to_string(),
            child.args().to_vec(),
            child.kargs().clone().into_inner(),
        ));

        let child_args = Args::new(child.args(), child.kargs().clone());
        func.invoke(&child_args, &child)
    }

    pub fn eval(&self, src: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let expr = self.env().compile_expression(src)?;
        Ok(expr.eval(Value::from_object(self.clone()))?)
    }

    pub fn render(&self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let tmpl = self.env().get_template(name)?;
        Ok(tmpl.render(Value::from_object(self.clone()))?)
    }

    pub fn render_str(&self, source: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.env().render_str(source, Value::from_object(self.clone()))?)
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

impl Reflect for Scope {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let name = key.as_str()?;

        if name == Self::KEY {
            return Some(Value::from_object(Self::clone(self)));
        }

        if name == "args" {
            return Some(Value::from_iter(self.args().iter().cloned()));
        }

        if let Some(value) = self.kargs().get(name) {
            return Some(value.clone());
        }

        let slot = self.get(name)?;

        match &*slot {
            Object::Value(value) => Some(value.clone()),
            Object::Func(func) => Some(Value::from_object(func.clone())),
            Object::Routine(rt) => Some(Value::from_object(rt.clone())),
        }
    }
}
