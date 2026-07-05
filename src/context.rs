use std::sync::Arc;

use crate::{Args, Diagnostic, Environment, Error, Object, Scope};

pub struct Context<'a> {
    trace_id: ulid::Ulid,
    args: Args,
    env: &'a Environment<'a>,
    scope: Scope,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Context<'a> {
    pub fn new(trace_id: ulid::Ulid, args: Args, env: &'a Environment<'a>, scope: Scope) -> Self {
        Self {
            trace_id,
            args,
            env,
            scope,
            diagnostics: vec![],
        }
    }

    pub fn trace_id(&self) -> &ulid::Ulid {
        &self.trace_id
    }

    pub fn args(&self) -> &Args {
        &self.args
    }

    pub fn env(&self) -> &Environment<'a> {
        self.env
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn emit(&mut self, diagnostic: Diagnostic) -> &mut Self {
        self.diagnostics.push(diagnostic);
        self
    }

    pub fn call(&mut self, name: impl AsRef<str>, args: impl Into<Args>) -> Result<(), Box<dyn std::error::Error>> {
        let name = name.as_ref();

        let object = self
            .scope
            .get(name)
            .ok_or_else(|| Error::action(self.trace_id, name, "action not found"))?;

        let action = {
            let guard = object.read().map_err(|_| Error::message("scope lock poisoned"))?;
            match &*guard {
                Object::Action(action) => Arc::clone(action),
                _ => return Err(Box::new(Error::action(self.trace_id, name, "action not found"))),
            }
        };

        let previous = std::mem::replace(&mut self.args, args.into());
        let result = action.invoke(self);
        self.args = previous;
        result
    }

    pub fn eval(&mut self, name: impl AsRef<str>, args: impl Into<Args>) -> Result<bool, Box<dyn std::error::Error>> {
        let name = name.as_ref();

        let object = self
            .scope
            .get(name)
            .ok_or_else(|| Error::action(self.trace_id, name, "predicate not found"))?;

        let predicate = {
            let guard = object.read().map_err(|_| Error::message("scope lock poisoned"))?;
            match &*guard {
                Object::Predicate(predicate) => Arc::clone(predicate),
                _ => return Err(Box::new(Error::action(self.trace_id, name, "predicate not found"))),
            }
        };

        let previous = std::mem::replace(&mut self.args, args.into());
        let result = predicate.invoke(self);
        self.args = previous;
        result
    }
}

impl<'a> std::ops::Deref for Context<'a> {
    type Target = Scope;

    fn deref(&self) -> &Self::Target {
        &self.scope
    }
}
