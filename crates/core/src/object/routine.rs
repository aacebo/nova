use std::sync::Arc;

use minijinja::value::Kwargs;

use crate::{Action, Args, KArgs, Object, Reflect, Scope, Step, Value, event};

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

    pub fn get(&self, key: &str) -> Option<Value> {
        let slot = self.scope.get(key)?;

        match &*slot {
            Object::Value(value) => Some(value.clone()),
            Object::Func(func) => Some(Value::from_object(func.clone())),
            Object::Routine(rt) => Some(Value::from_object(rt.clone())),
        }
    }
}

impl std::fmt::Debug for Routine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Routine").field("name", &self.name).finish_non_exhaustive()
    }
}

impl Reflect for Routine {
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

        let args = Args::new(positional, kargs);
        self.invoke(&args, &caller)
            .map_err(|err| minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, err.to_string()))?;

        Ok(Value::from(()))
    }
}

impl Action for Routine {
    fn invoke(&self, args: &Args, _scope: &Scope) -> Result<(), Box<dyn std::error::Error>> {
        let child = self.scope.fork(&self.name, args.args().to_vec(), args.kargs().clone());
        let total = self.steps.len();

        for (index, step) in self.steps.iter().enumerate() {
            let name = step.name.clone().unwrap_or_default();
            child.dispatch(event::step::start(&self.name, &name, index, total));

            let skipped = step
                .cond
                .as_ref()
                .map(|cond| !child.eval(cond).map(|v| v.is_true()).unwrap_or(false))
                .unwrap_or(false);

            if skipped {
                child.dispatch(event::step::end(
                    &self.name,
                    &name,
                    index,
                    event::step::Status::Skipped,
                    std::time::Duration::ZERO,
                ));
                continue;
            }

            let started = std::time::Instant::now();
            let step_args = Args::new(child.args(), child.kargs().clone());
            let result = step.invoke(&step_args, &child);
            let elapsed = started.elapsed();
            let status = if result.is_err() {
                event::step::Status::Error
            } else {
                event::step::Status::Ok
            };

            child.dispatch(event::step::end(&self.name, &name, index, status, elapsed));
            result?;
        }

        Ok(())
    }
}
