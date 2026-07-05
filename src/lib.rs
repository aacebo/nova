use std::sync::Arc;

mod arena;
mod args;
pub mod builtin;
mod context;
mod diagnostic;
mod error;
mod object;
mod routine;
mod scope;
mod span;

pub use arena::*;
pub use args::*;
pub use context::*;
pub use diagnostic::*;
pub use error::*;
pub use minijinja::context;
pub use object::*;
pub use routine::*;
pub use scope::*;
pub use span::*;

pub type Value = minijinja::Value;
pub type Environment<'a> = minijinja::Environment<'a>;

pub trait Action: Send + Sync {
    fn invoke(&self, ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Predicate: Send + Sync {
    fn invoke(&self, ctx: &Context) -> Result<bool, Box<dyn std::error::Error>>;
}

pub struct Runtime<'a> {
    env: Environment<'a>,
    scope: Scope,
}

impl<'a> Runtime<'a> {
    pub fn env(&self) -> &Environment<'a> {
        &self.env
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    /// Top-level entry point: invoke a registered action by name.
    ///
    /// Resolves `name` against the runtime's scope, forks a fresh scope for the call,
    /// and mints a new `trace_id` for the resulting invocation.
    pub fn invoke(&self, name: &str, args: impl Into<Args>) -> Result<(), Box<dyn std::error::Error>> {
        let trace_id = ulid::Ulid::new();
        let object = self
            .scope
            .get(name)
            .ok_or_else(|| Error::action(trace_id, name, "action not found"))?;

        let action = {
            let guard = object.read().map_err(|_| Error::message("scope lock poisoned"))?;
            match &*guard {
                Object::Action(action) => Arc::clone(action),
                _ => return Err(Box::new(Error::action(trace_id, name, "action not found"))),
            }
        };

        let mut ctx = Context::new(trace_id, args.into(), &self.env, self.scope.fork());
        action.invoke(&mut ctx)
    }
}

#[doc(hidden)]
pub struct Builder<'a> {
    env: Environment<'a>,
    scope: Scope,
}

impl<'a> Default for Builder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Builder<'a> {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
            scope: Scope::from(Arena::new()),
        }
    }

    pub fn var(self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        let name = name.into();
        let value = value.into();
        self.scope.set(name.clone(), Var::new(name, value));
        self
    }

    pub fn action(self, name: impl Into<String>, action: impl Action + 'static) -> Self {
        self.scope.set(name, action);
        self
    }

    pub fn predicate(self, name: impl Into<String>, predicate: impl Predicate + 'static) -> Self {
        self.scope.set(name, Object::predicate(predicate));
        self
    }

    pub fn build(self) -> Runtime<'a> {
        Runtime {
            env: self.env,
            scope: self.scope,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[test]
    fn runtime_invokes_action_by_name() {
        static CALLS: AtomicUsize = AtomicUsize::new(0);

        struct Bump;

        impl Action for Bump {
            fn invoke(&self, _ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
                CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        CALLS.store(0, Ordering::SeqCst);
        let runtime = Builder::new().action("bump", Bump).build();
        runtime.invoke("bump", Args::new()).unwrap();
        assert_eq!(CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn runtime_invoke_unknown_name_errors() {
        let runtime = Builder::new().build();
        assert!(runtime.invoke("missing", Args::new()).is_err());
    }

    #[test]
    fn context_call_chains_with_fresh_args_and_restores() {
        static SEEN: AtomicUsize = AtomicUsize::new(0);

        // the chained action asserts it sees the args it was called with
        struct Callee;

        impl Action for Callee {
            fn invoke(&self, ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
                assert_eq!(ctx.args().get("n"), Some(&Value::from(2)));
                SEEN.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        // the caller chains into "callee" with new args, then checks its own args survived
        struct Caller;

        impl Action for Caller {
            fn invoke(&self, ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
                assert_eq!(ctx.args().get("n"), Some(&Value::from(1)));
                ctx.call("callee", [("n", 2)])?;
                assert_eq!(ctx.args().get("n"), Some(&Value::from(1)));
                Ok(())
            }
        }

        SEEN.store(0, Ordering::SeqCst);
        let runtime = Builder::new().action("caller", Caller).action("callee", Callee).build();

        runtime.invoke("caller", [("n", 1)]).unwrap();
        assert_eq!(SEEN.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn context_eval_evaluates_predicate_by_name() {
        struct Always(bool);

        impl Predicate for Always {
            fn invoke(&self, _ctx: &Context) -> Result<bool, Box<dyn std::error::Error>> {
                Ok(self.0)
            }
        }

        struct Probe;

        impl Action for Probe {
            fn invoke(&self, ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
                assert!(ctx.eval("yes", Args::new())?);
                assert!(!ctx.eval("no", Args::new())?);
                assert!(ctx.eval("missing", Args::new()).is_err());
                Ok(())
            }
        }

        let runtime = Builder::new()
            .predicate("yes", Always(true))
            .predicate("no", Always(false))
            .action("probe", Probe)
            .build();

        runtime.invoke("probe", Args::new()).unwrap();
    }

    #[test]
    fn if_builtin_dispatches_by_name() {
        static THEN: AtomicUsize = AtomicUsize::new(0);
        static ELSE: AtomicUsize = AtomicUsize::new(0);

        struct GtZero;

        impl Predicate for GtZero {
            fn invoke(&self, ctx: &Context) -> Result<bool, Box<dyn std::error::Error>> {
                Ok(ctx.args().get("n") > Some(&Value::from(0)))
            }
        }

        struct Then;

        impl Action for Then {
            fn invoke(&self, _ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
                THEN.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        struct Else;

        impl Action for Else {
            fn invoke(&self, _ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
                ELSE.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        THEN.store(0, Ordering::SeqCst);
        ELSE.store(0, Ordering::SeqCst);
        let runtime = Builder::new()
            .predicate("gt_zero", GtZero)
            .action("then", Then)
            .action("else", Else)
            .action("branch", builtin::If::new("gt_zero", "then").or_else("else"))
            .build();

        runtime.invoke("branch", [("n", 1)]).unwrap();
        runtime.invoke("branch", [("n", 0)]).unwrap();
        assert_eq!(THEN.load(Ordering::SeqCst), 1);
        assert_eq!(ELSE.load(Ordering::SeqCst), 1);
    }
}
