use crate::{Action, Args, Context, Environment, Error, Scope};

pub struct Routine<'a> {
    env: Environment<'a>,
    entrypoint: Option<ulid::Ulid>,
}

impl<'a> Default for Routine<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Routine<'a> {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
            entrypoint: None,
        }
    }

    pub fn invoke(&self, args: impl Into<Args>, scope: &Scope) -> Result<(), Box<dyn std::error::Error>> {
        let trace_id = ulid::Ulid::new();
        let mut ctx = Context::new(trace_id, args.into(), &self.env, scope.fork());
        Action::invoke(self, &mut ctx)?;
        Ok(())
    }
}

impl<'a> Action for Routine<'a> {
    fn invoke(&self, ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(id) = self.entrypoint
            && let Some(object) = ctx.get(id.to_string())
        {
            let object = object.read().unwrap();

            if let Some(action) = object.as_action() {
                action.invoke(ctx)?;
            } else {
                return Err(Box::new(Error::action(*ctx.trace_id(), id, "action not found")));
            }
        }

        Ok(())
    }
}
