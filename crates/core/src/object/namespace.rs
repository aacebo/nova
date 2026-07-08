use std::sync::Arc;

use minijinja::value::Kwargs;

use crate::{Args, Function, Object, Scope, Value};

#[derive(Clone)]
pub struct Namespace {
    name: String,
    scope: Scope,
    entrypoint: Function,
}

impl Namespace {
    pub fn new(name: impl Into<String>, scope: Scope) -> Self {
        let name = name.into();
        let entrypoint = {
            let name = name.to_string();
            let scope = scope.clone();

            Function::action(name.clone(), move |args: &Args| {
                scope.call(&name, args.clone())?;
                Ok(())
            })
        };

        Self {
            name,
            scope,
            entrypoint,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn entrypoint(&self) -> &Function {
        &self.entrypoint
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        let slot = self.scope.get(key)?;

        match &*slot {
            Object::Var(var) => Some(var.value.clone()),
            Object::Func(func) => Some(Value::from_object(func.clone())),
            Object::Namespace(ns) => Some(Value::from_object(ns.clone())),
            _ => None,
        }
    }
}

impl std::fmt::Debug for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Namespace").field("name", &self.name).finish_non_exhaustive()
    }
}

impl minijinja::value::Object for Namespace {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        self.get(key.as_str()?)
    }

    fn call(self: &Arc<Self>, state: &minijinja::State<'_, '_>, args: &[Value]) -> Result<Value, minijinja::Error> {
        let (positional, kwargs): (&[Value], Kwargs) = minijinja::value::from_args(args)?;
        let caller = state
            .lookup(Scope::KEY)
            .and_then(|v| v.downcast_object::<Scope>())
            .ok_or_else(|| minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, "no scope bound to template render"))?;

        let mut args = Args::from_kwargs(kwargs)?;

        for value in positional {
            args.push(value.clone());
        }

        let value = self
            .entrypoint
            .invoke(&args)
            .map_err(|err| minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, err.to_string()))?;

        caller.merge(&self.name, &self.scope);
        Ok(value.unwrap_or(Value::UNDEFINED))
    }
}
