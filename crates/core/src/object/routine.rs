use std::sync::Arc;

use minijinja::value::Kwargs;

use crate::{Action, KArgs, Object, Scope, Step, Value};

#[derive(Clone)]
pub struct Routine {
    name: String,
    scope: Scope,
    steps: Vec<Step>,
}

impl Routine {
    pub fn new(name: impl Into<String>, scope: Scope, steps: impl IntoIterator<Item = impl Into<Step>>) -> Self {
        Self {
            name: name.into(),
            scope,
            steps: steps.into_iter().map(Into::into).collect(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    pub fn run(&self, caller: &Scope, args: &[Value], kargs: &KArgs) -> Result<(), Box<dyn std::error::Error>> {
        let child = self.scope.fork(&self.name, args.to_vec(), kargs.clone());

        for step in &self.steps {
            step.invoke(child.args(), child.kargs(), &child)?;
        }

        caller.merge(&child);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        let slot = self.scope.get(key)?;

        match &*slot {
            Object::Var(var) => Some(var.value.clone()),
            Object::Func(func) => Some(Value::from_object(func.clone())),
            Object::Routine(rt) => Some(Value::from_object(rt.clone())),
            _ => None,
        }
    }
}

impl std::fmt::Debug for Routine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Routine").field("name", &self.name).finish_non_exhaustive()
    }
}

impl minijinja::value::Object for Routine {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        self.get(key.as_str()?)
    }

    fn call(self: &Arc<Self>, state: &minijinja::State<'_, '_>, args: &[Value]) -> Result<Value, minijinja::Error> {
        let (positional, kwargs): (&[Value], Kwargs) = minijinja::value::from_args(args)?;
        let kargs = KArgs::from_kwargs(kwargs)?;
        let caller = state
            .lookup(Scope::KEY)
            .and_then(|v| v.downcast_object::<Scope>())
            .ok_or_else(|| minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, "no scope bound to template render"))?;

        self.run(caller.as_ref(), positional, &kargs)
            .map_err(|err| minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, err.to_string()))?;

        Ok(Value::UNDEFINED)
    }
}
