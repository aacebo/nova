use std::sync::Arc;

use nova_reflect::Value;

use crate::{Action, Args, Binding, Call, Context, Error, Predicate};

#[derive(Clone)]
pub struct Function {
    name: String,
    callback: Callback,
}

impl Function {
    pub fn new(name: impl Into<String>, callback: Callback) -> Self {
        Self {
            name: name.into(),
            callback,
        }
    }

    pub fn action(name: impl Into<String>, action: impl Action + 'static) -> Self {
        Self::new(name, Callback::action(action))
    }

    pub fn predicate(name: impl Into<String>, predicate: impl Predicate + 'static) -> Self {
        Self::new(name, Callback::predicate(predicate))
    }

    pub fn func(name: impl Into<String>, func: impl crate::Func + 'static) -> Self {
        Self::new(name, Callback::func(func))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn callback(&self) -> &Callback {
        &self.callback
    }

    pub fn callback_mut(&mut self) -> &mut Callback {
        &mut self.callback
    }

    pub fn invoke(&self, args: &Args, ctx: &dyn Context) -> Result<Binding, Box<dyn std::error::Error>> {
        self.callback.invoke(args, ctx)
    }
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function").field("name", &self.name).finish_non_exhaustive()
    }
}

impl Call for Function {
    fn call(&self, args: &Args, ctx: &dyn Context) -> Result<Binding, Error> {
        let child = ctx.fork(&self.name, args.args().to_vec(), args.kargs().clone());
        let child_args = Args::new(child.args().to_vec(), child.kargs().clone());

        self.callback
            .invoke(&child_args, child.as_ref())
            .map_err(|err| Error::message(err.to_string()))
    }
}

impl From<Function> for Binding {
    fn from(value: Function) -> Self {
        Binding::callable(value)
    }
}

#[derive(Clone)]
pub enum Callback {
    Action(Arc<dyn Action>),
    Predicate(Arc<dyn Predicate>),
    Func(Arc<dyn crate::Func>),
}

impl Callback {
    pub fn action(action: impl Action + 'static) -> Self {
        Self::Action(Arc::new(action))
    }

    pub fn predicate(predicate: impl Predicate + 'static) -> Self {
        Self::Predicate(Arc::new(predicate))
    }

    pub fn func(func: impl crate::Func + 'static) -> Self {
        Self::Func(Arc::new(func))
    }

    pub fn invoke(&self, args: &Args, ctx: &dyn Context) -> Result<Binding, Box<dyn std::error::Error>> {
        match self {
            Self::Action(action) => {
                action.invoke(args, ctx)?;
                Ok(Binding::new(Value::Null))
            }
            Self::Predicate(predicate) => Ok(Binding::new(Value::Bool(predicate.invoke(args, ctx)?))),
            Self::Func(func) => func.invoke(args, ctx),
        }
    }
}
