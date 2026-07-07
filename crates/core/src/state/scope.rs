use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{Arena, Args, Diagnostic, Entry, Object, Slot, SlotMut, Traced};

/// The live diagnostic buffer shared across a scope and its forks.
pub type Diagnostics = Arc<Mutex<Vec<Diagnostic>>>;

#[derive(Default, Clone)]
pub struct Scope(Arc<_Scope>);

#[derive(Default)]
struct _Scope {
    trace_id: ulid::Ulid,
    parent: Option<Scope>,
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
            symbols: Default::default(),
            arena: Default::default(),
            args: Default::default(),
            diagnostics: Default::default(),
        }))
    }

    pub fn trace_id(&self) -> &ulid::Ulid {
        &self.0.trace_id
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
}

impl Traced for Scope {
    fn trace_id(&self) -> ulid::Ulid {
        self.0.trace_id
    }
}

impl From<Arena> for Scope {
    fn from(value: Arena) -> Self {
        Self(Arc::new(_Scope {
            trace_id: ulid::Ulid::new(),
            parent: None,
            arena: Arc::new(Mutex::new(value)),
            symbols: Default::default(),
            args: Default::default(),
            diagnostics: Default::default(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, Value, Var};

    #[test]
    fn set_then_get_returns_object() {
        let scope = Scope::new();
        scope.set("x", Var::new("x", 1));

        let object = scope.get("x").expect("x should be set");
        let value = object.as_var().unwrap().value.clone();
        assert_eq!(value, Value::from(1));
        assert!(scope.has("x"));
        assert_eq!(scope.len(), 1);
    }

    #[test]
    fn get_mut_mutates_in_place() {
        let scope = Scope::new();
        scope.set("x", Var::new("x", 1));

        {
            let mut slot = scope.get_mut("x").expect("x should be set");
            slot.as_var_mut().unwrap().value = Value::from(9);
        }

        let value = scope.get("x").unwrap().as_var().unwrap().value.clone();
        assert_eq!(value, Value::from(9));
    }

    #[test]
    fn set_twice_updates_in_place() {
        let scope = Scope::new();
        scope.set("x", Var::new("x", 1));
        scope.set("x", Var::new("x", 2));

        assert_eq!(scope.len(), 1);
        assert_eq!(scope.0.arena.lock().unwrap().len(), 1);

        let object = scope.get("x").unwrap();
        let value = object.as_var().unwrap().value.clone();
        assert_eq!(value, Value::from(2));
    }

    #[test]
    fn fork_child_resolves_parent_binding() {
        let parent = Scope::new();
        parent.set("x", Var::new("x", 1));

        let child = parent.fork(Args::new());
        let object = child.get("x").expect("child should resolve parent's x");
        let value = object.as_var().unwrap().value.clone();
        assert_eq!(value, Value::from(1));
    }

    #[test]
    fn child_set_on_existing_parent_key_updates_parent() {
        let parent = Scope::new();
        parent.set("x", Var::new("x", 1));

        let child = parent.fork(Args::new());
        child.set("x", Var::new("x", 2));

        let from_parent = parent.get("x").unwrap();
        let parent_value = from_parent.as_var().unwrap().value.clone();
        assert_eq!(parent_value, Value::from(2));

        assert!(child.0.symbols.lock().unwrap().is_empty());
    }

    #[test]
    fn child_set_new_key_lands_at_root() {
        let root = Scope::new();
        let child = root.fork(Args::new()).fork(Args::new());
        child.set("y", Var::new("y", 7));

        assert!(!root.0.symbols.lock().unwrap().is_empty());
        assert!(child.0.symbols.lock().unwrap().is_empty());

        let object = child.get("y").unwrap();
        let value = object.as_var().unwrap().value.clone();
        assert_eq!(value, Value::from(7));
    }

    #[test]
    fn del_removes_binding() {
        let scope = Scope::new();
        scope.set("x", Var::new("x", 1));
        scope.del("x");

        assert!(!scope.has("x"));
        assert!(scope.get("x").is_none());
        assert!(scope.0.symbols.lock().unwrap().is_empty());
        assert!(scope.0.arena.lock().unwrap().is_empty());
    }

    #[test]
    fn child_del_recurses_to_parent() {
        let parent = Scope::new();
        parent.set("x", Var::new("x", 1));

        let child = parent.fork(Args::new());
        child.del("x");

        assert!(!parent.has("x"));
        assert!(parent.get("x").is_none());
    }

    #[test]
    fn action_is_resolvable_through_scope() {
        let scope = Scope::new();
        let id = ulid::Ulid::new();
        scope.set(
            id.to_string(),
            Object::action(
                id.to_string(),
                |_ctx: &mut Context| -> Result<(), Box<dyn std::error::Error>> { Ok(()) },
            ),
        );

        let object = scope.get(id.to_string()).expect("action should be registered");
        assert!(object.is_func());
    }
}
