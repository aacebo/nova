use std::sync::Arc;

use nova_core::{Action, Args, Binding, Call, Context, Error, Namespace, Step, event};
use nova_reflect::Value;

use crate::Scope;

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

    pub fn get(&self, key: &str) -> Option<Binding> {
        let slot = self.scope.get(key)?;
        Some((*slot).clone())
    }

    fn validate(&self, args: &Args, ctx: &dyn Context) -> Result<(), Box<dyn std::error::Error>> {
        let Some(validator) = &self.validator else {
            return Ok(());
        };

        let ty = nova_reflect::MapType::new(nova_reflect::Type::Any, nova_reflect::Type::Any, nova_reflect::Type::Any);
        let mut instance = nova_reflect::Map::new(&ty);

        for (key, value) in args.iter() {
            instance.insert(key, value.to_owned());
        }

        let instance = nova_reflect::Value::Map(instance);

        if let Err(err) = nova_schema::Validate::validate(validator.as_ref(), &instance) {
            let message = format!("invalid args for `{}`: {}", self.name, err);
            return Err(Box::new(Error::action(ctx.trace_id().to_string(), &self.name, message)));
        }

        Ok(())
    }
}

impl std::fmt::Debug for Routine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Routine").field("name", &self.name).finish_non_exhaustive()
    }
}

impl Namespace for Routine {
    fn member(&self, name: &str) -> Option<Binding> {
        self.get(name)
    }

    fn members(&self) -> Vec<String> {
        Context::names(&self.scope)
    }
}

impl Call for Routine {
    fn call(&self, args: &Args, ctx: &dyn Context) -> Result<Binding, Error> {
        self.invoke(args, ctx).map_err(|err| Error::message(err.to_string()))?;
        Ok(Binding::new(Value::Null))
    }
}

impl Action for Routine {
    fn invoke(&self, args: &Args, ctx: &dyn Context) -> Result<(), Box<dyn std::error::Error>> {
        self.validate(args, ctx)?;

        let child = self.scope.fork(&self.name, args.args().to_vec(), args.kargs().clone());
        let total = self.steps.len();

        for (index, step) in self.steps.iter().enumerate() {
            let name = step.name.clone().unwrap_or_default();
            child.dispatch(event::step::start(&self.name, &name, index, total));

            let skipped = step
                .cond
                .as_ref()
                .map(|cond| !child.eval(cond).map(|v| v.is_truthy()).unwrap_or(false))
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
            let result = crate::step::invoke(step, &child);
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

impl From<Routine> for Binding {
    fn from(value: Routine) -> Self {
        Binding::callable_namespace(value)
    }
}
