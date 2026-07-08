use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{Arena, Args, Diagnostic, Entry, Environment, Object, Slot, SlotMut, Traced, Value};

pub type Diagnostics = Arc<Mutex<Vec<Diagnostic>>>;

#[derive(Default, Clone)]
pub struct Scope(Arc<_Scope>);

#[derive(Default)]
struct _Scope {
    trace_id: ulid::Ulid,
    parent: Option<Scope>,
    env: Arc<Environment<'static>>,
    symbols: Mutex<HashMap<String, ulid::Ulid>>,
    arena: Arc<Mutex<Arena>>,
    args: Args,
    diagnostics: Diagnostics,
}

impl Scope {
    pub fn new() -> Self {
        Self(Arc::new(_Scope {
            trace_id: ulid::Ulid::new(),
            parent: None,
            env: Default::default(),
            symbols: Default::default(),
            arena: Default::default(),
            args: Default::default(),
            diagnostics: Default::default(),
        }))
    }

    pub fn with_env(self, env: Environment<'static>) -> Self {
        let mut inner = Arc::try_unwrap(self.0).unwrap_or_else(|_| panic!("with_env on a shared scope"));
        inner.env = Arc::new(env);
        Self(Arc::new(inner))
    }

    pub fn trace_id(&self) -> &ulid::Ulid {
        &self.0.trace_id
    }

    pub fn env(&self) -> &Environment<'static> {
        &self.0.env
    }

    pub fn args(&self) -> &Args {
        &self.0.args
    }

    pub fn diagnostics(&self) -> &Diagnostics {
        &self.0.diagnostics
    }

    pub fn emit(&self, diagnostic: Diagnostic) -> &Self {
        self.0.diagnostics.lock().unwrap().push(diagnostic);
        self
    }

    pub fn take_diagnostics(&self) -> Vec<Diagnostic> {
        self.0.diagnostics.lock().unwrap().drain(..).collect()
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

    pub fn fork(&self, args: impl Into<Args>) -> Self {
        Self(Arc::new(_Scope {
            trace_id: self.0.trace_id,
            parent: Some(self.clone()),
            env: self.0.env.clone(),
            symbols: Default::default(),
            arena: self.0.arena.clone(),
            args: args.into(),
            diagnostics: Default::default(),
        }))
    }

    pub fn merge(&self, name: &str, child: &Scope) {
        let diagnostics = child.take_diagnostics();

        if diagnostics.is_empty() {
            return;
        }

        let mut node = Diagnostic::new(self.0.trace_id).message(name);
        node.children = diagnostics;
        self.0.diagnostics.lock().unwrap().push(node);
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
            self.0.arena.lock().unwrap().set(&id, object.into());
        } else if let Some(parent) = &self.0.parent {
            parent.set(key, object);
        } else {
            let id = ulid::Ulid::new();
            self.0.symbols.lock().unwrap().insert(key, id);
            self.0.arena.lock().unwrap().set(&id, object.into());
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

    pub fn call(&self, name: impl AsRef<str>, args: impl Into<Args>) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        let name = name.as_ref();
        let func = self.get_func(name)?;
        let child = self.fork(args);
        let result = {
            let _guard = crate::enter(&child);
            func.invoke(child.args())
        };
        self.merge(name, &child);
        result
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
}

impl Traced for Scope {
    fn trace_id(&self) -> ulid::Ulid {
        self.0.trace_id
    }
}

impl Scope {
    pub const KEY: &'static str = "__$scope__";
}

impl std::fmt::Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scope")
            .field("trace_id", &self.0.trace_id)
            .finish_non_exhaustive()
    }
}

impl minijinja::value::Object for Scope {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let name = key.as_str()?;

        if name == Self::KEY {
            return Some(Value::from_object(Self::clone(self)));
        }

        if let Some(value) = self.args().get(name) {
            return Some(value.clone());
        }

        let slot = self.get(name)?;

        match &*slot {
            Object::Var(var) => Some(var.value.clone()),
            Object::Func(func) => Some(Value::from_object(func.clone())),
            Object::Namespace(ns) => Some(Value::from_object(ns.clone())),
            _ => None,
        }
    }
}

impl From<Arena> for Scope {
    fn from(value: Arena) -> Self {
        Self(Arc::new(_Scope {
            trace_id: ulid::Ulid::new(),
            parent: None,
            env: Default::default(),
            arena: Arc::new(Mutex::new(value)),
            symbols: Default::default(),
            args: Default::default(),
            diagnostics: Default::default(),
        }))
    }
}
