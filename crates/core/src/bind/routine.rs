use std::sync::Arc;

use nova_reflect::Value;
use nova_template::{Args, Pointer};

use crate::{Action, Binding, Error, Scope, Step, event};

#[derive(Clone)]
pub struct Routine {
    name: String,
    scope: Scope,
    steps: Vec<Step>,
    validator: Option<Arc<nova_schema::Schema>>,
}

impl Routine {
    pub fn new(
        name: impl Into<String>,
        scope: Scope,
        steps: impl IntoIterator<Item = impl Into<Step>>,
        validator: Option<Arc<nova_schema::Schema>>,
    ) -> Self {
        Self {
            name: name.into(),
            scope,
            steps: steps.into_iter().map(Into::into).collect(),
            validator,
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

    pub fn get(&self, key: &str) -> Option<Pointer> {
        let slot = self.scope.get(key)?;

        match &*slot {
            Binding::Value(value) => Some(value.clone()),
            Binding::Func(func) => Some(Pointer::callable(func.clone())),
            Binding::Routine(rt) => Some(Pointer::callable_namespace(rt.clone())),
        }
    }

    fn validate(&self, args: &Args, scope: &Scope) -> Result<(), Box<dyn std::error::Error>> {
        let Some(validator) = &self.validator else {
            return Ok(());
        };

        let ty = nova_reflect::MapType::new(nova_reflect::Type::Any, nova_reflect::Type::Any, nova_reflect::Type::Any);
        let mut instance = nova_reflect::Map::new(&ty);

        for (key, value) in args.iter() {
            instance.insert(key.into_owned(), value.into_owned());
        }

        let instance = nova_reflect::Value::Map(instance);

        if let Err(err) = nova_schema::Validate::validate(validator.as_ref(), &instance) {
            let message = format!("invalid args for `{}`: {}", self.name, err);
            return Err(Box::new(Error::action(scope.trace_id().to_string(), &self.name, message)));
        }

        Ok(())
    }
}

impl std::fmt::Debug for Routine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Routine").field("name", &self.name).finish_non_exhaustive()
    }
}

impl nova_template::Namespace for Routine {
    fn member(&self, name: &str) -> Option<Pointer> {
        self.get(name)
    }

    fn members(&self) -> Vec<String> {
        nova_template::Context::names(&self.scope)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl nova_template::Call for Routine {
    fn call(&self, args: &Args) -> Result<Pointer, nova_template::Error> {
        let caller = args
            .caller_as::<Scope>()
            .ok_or_else(|| nova_template::Error::message("no scope bound to template render"))?;

        self.invoke(args, caller)
            .map_err(|err| nova_template::Error::message(err.to_string()))?;

        Ok(Pointer::new(Value::Null))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Action for Routine {
    fn invoke(&self, args: &Args, scope: &Scope) -> Result<(), Box<dyn std::error::Error>> {
        self.validate(args, scope)?;

        let child = self.scope.fork(&self.name, args.args().to_vec(), args.kargs().clone());
        let total = self.steps.len();

        for (index, step) in self.steps.iter().enumerate() {
            let name = step.name.clone().unwrap_or_default();
            child.dispatch(event::step::start(&self.name, &name, index, total));

            let skipped = step
                .cond
                .as_ref()
                .map(|cond| {
                    !child
                        .eval(cond)
                        .map(|v| nova_template::is_truthy(&v.value()))
                        .unwrap_or(false)
                })
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
            let step_args = Args::new(child.args().to_vec(), child.kargs().clone());
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
