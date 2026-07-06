use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use crate::{Arena, Object};

#[derive(Default, Clone)]
pub struct Scope(Arc<_Scope>);

#[derive(Default)]
struct _Scope {
    parent: Option<Scope>,
    symbols: Mutex<HashMap<String, ulid::Ulid>>,
    arena: Arc<Mutex<Arena>>,
}

impl Scope {
    pub fn new() -> Self {
        Self(Arc::new(_Scope {
            parent: None,
            symbols: Default::default(),
            arena: Default::default(),
        }))
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

    pub fn fork(&self) -> Self {
        Self(Arc::new(_Scope {
            parent: Some(self.clone()),
            symbols: Default::default(),
            arena: self.0.arena.clone(),
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

    pub fn get(&self, key: impl AsRef<str>) -> Option<Arc<RwLock<Object>>> {
        let id = self.0.symbols.lock().unwrap().get(key.as_ref()).copied();

        if let Some(id) = id
            && let Some(object) = self.0.arena.lock().unwrap().get(&id)
        {
            Some(object)
        } else if let Some(parent) = &self.0.parent {
            parent.get(key)
        } else {
            None
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

impl From<Arena> for Scope {
    fn from(value: Arena) -> Self {
        Self(Arc::new(_Scope {
            parent: None,
            arena: Arc::new(Mutex::new(value)),
            symbols: Default::default(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, Context, Value, Var};

    #[test]
    fn set_then_get_returns_object() {
        let scope = Scope::new();
        scope.set("x", Var::new("x", 1));

        let object = scope.get("x").expect("x should be set");
        let value = object.read().unwrap().as_var().unwrap().value.clone();
        assert_eq!(value, Value::from(1));
        assert!(scope.has("x"));
        assert_eq!(scope.len(), 1);
    }

    #[test]
    fn set_twice_updates_in_place() {
        let scope = Scope::new();
        scope.set("x", Var::new("x", 1));
        scope.set("x", Var::new("x", 2));

        assert_eq!(scope.len(), 1);
        assert_eq!(scope.0.arena.lock().unwrap().len(), 1);

        let object = scope.get("x").unwrap();
        let value = object.read().unwrap().as_var().unwrap().value.clone();
        assert_eq!(value, Value::from(2));
    }

    #[test]
    fn fork_child_resolves_parent_binding() {
        let parent = Scope::new();
        parent.set("x", Var::new("x", 1));

        let child = parent.fork();
        let object = child.get("x").expect("child should resolve parent's x");
        let value = object.read().unwrap().as_var().unwrap().value.clone();
        assert_eq!(value, Value::from(1));
    }

    #[test]
    fn child_set_on_existing_parent_key_updates_parent() {
        let parent = Scope::new();
        parent.set("x", Var::new("x", 1));

        let child = parent.fork();
        child.set("x", Var::new("x", 2));

        let from_parent = parent.get("x").unwrap();
        let parent_value = from_parent.read().unwrap().as_var().unwrap().value.clone();
        assert_eq!(parent_value, Value::from(2));

        assert!(child.0.symbols.lock().unwrap().is_empty());
    }

    #[test]
    fn child_set_new_key_lands_at_root() {
        let root = Scope::new();
        let child = root.fork().fork();
        child.set("y", Var::new("y", 7));

        assert!(!root.0.symbols.lock().unwrap().is_empty());
        assert!(child.0.symbols.lock().unwrap().is_empty());

        let object = child.get("y").unwrap();
        let value = object.read().unwrap().as_var().unwrap().value.clone();
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

        let child = parent.fork();
        child.del("x");

        assert!(!parent.has("x"));
        assert!(parent.get("x").is_none());
    }

    #[test]
    fn action_is_resolvable_through_scope() {
        struct Noop;

        impl Action for Noop {
            fn invoke(&self, _ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
                Ok(())
            }
        }

        let scope = Scope::new();
        let id = ulid::Ulid::new();
        scope.set(id.to_string(), Noop);

        let object = scope.get(id.to_string()).expect("action should be registered");
        assert!(object.read().unwrap().is_action());
    }
}
