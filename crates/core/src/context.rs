use std::sync::Arc;

use crate::{Args, Diagnostic, Environment, Error, Object, Output, Scope, Value};

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

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
        self.diagnostics.drain(..).collect()
    }

    pub fn emit(&mut self, diagnostic: Diagnostic) -> &mut Self {
        self.diagnostics.push(diagnostic);
        self
    }

    fn child(&self, args: impl Into<Args>) -> Self {
        Self {
            trace_id: self.trace_id,
            args: args.into(),
            env: self.env,
            scope: self.scope.fork(),
            diagnostics: vec![],
        }
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

        let mut ctx = self.child(args);
        let result = action.invoke(&mut ctx);
        let output = Output::from(ctx);

        if !output.diagnostics.is_empty() {
            let mut node = Diagnostic::new(output.trace_id).message(name);
            node.id = output.id;
            node.children = output.diagnostics;
            self.diagnostics.push(node);
        }

        result
    }

    pub fn eval(&self, name: impl AsRef<str>, args: impl Into<Args>) -> Result<bool, Box<dyn std::error::Error>> {
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

        let ctx = self.child(args);
        predicate.invoke(&ctx)
    }

    pub fn map(&mut self, name: impl AsRef<str>, args: impl Into<Args>) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        let name = name.as_ref();
        let object = self
            .scope
            .get(name)
            .ok_or_else(|| Error::action(self.trace_id, name, "map not found"))?;

        let map = {
            let guard = object.read().map_err(|_| Error::message("scope lock poisoned"))?;
            match &*guard {
                Object::Map(map) => Arc::clone(map),
                _ => return Err(Box::new(Error::action(self.trace_id, name, "map not found"))),
            }
        };

        let mut ctx = self.child(args);
        let result = map.invoke(&mut ctx);
        let output = Output::from(ctx);

        if !output.diagnostics.is_empty() {
            let mut node = Diagnostic::new(output.trace_id).message(name);
            node.id = output.id;
            node.children = output.diagnostics;
            self.diagnostics.push(node);
        }

        result
    }
}

impl<'a> std::ops::Deref for Context<'a> {
    type Target = Scope;

    fn deref(&self) -> &Self::Target {
        &self.scope
    }
}
