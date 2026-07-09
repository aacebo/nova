use std::sync::Arc;

use minijinja::value::Kwargs;

use crate::{Action, Args, Call, KArgs, Predicate, Reflect, Scope, Value};

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

    pub fn func(name: impl Into<String>, func: impl Call + 'static) -> Self {
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

    pub fn invoke(&self, args: &Args, scope: &Scope) -> Result<Value, Box<dyn std::error::Error>> {
        self.callback.invoke(args, scope)
    }
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function").field("name", &self.name).finish_non_exhaustive()
    }
}

impl Reflect for Function {
    fn call(self: &Arc<Self>, state: &minijinja::State<'_, '_>, args: &[Value]) -> Result<Value, minijinja::Error> {
        let (positional, kwargs): (&[Value], Kwargs) = minijinja::value::from_args(args)?;
        let kargs = KArgs::from_kwargs(kwargs)?;
        let scope = state
            .lookup(Scope::KEY)
            .and_then(|v| v.downcast_object::<Scope>())
            .ok_or_else(|| minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, "no scope bound to template render"))?;

        let child = scope.fork(&self.name, positional.to_vec(), kargs);
        let args = Args::new(child.args(), child.kargs().clone());
        self.callback
            .invoke(&args, &child)
            .map_err(|err| minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, err.to_string()))
    }
}

#[derive(Clone)]
pub enum Callback {
    Action(Arc<dyn Action>),
    Predicate(Arc<dyn Predicate>),
    Func(Arc<dyn Call>),
}

impl Callback {
    pub fn action(action: impl Action + 'static) -> Self {
        Self::Action(Arc::new(action))
    }

    pub fn predicate(predicate: impl Predicate + 'static) -> Self {
        Self::Predicate(Arc::new(predicate))
    }

    pub fn func(func: impl Call + 'static) -> Self {
        Self::Func(Arc::new(func))
    }

    pub fn invoke(&self, args: &Args, scope: &Scope) -> Result<Value, Box<dyn std::error::Error>> {
        match self {
            Self::Action(action) => {
                action.invoke(args, scope)?;
                Ok(Value::from(()))
            }
            Self::Predicate(predicate) => Ok(Value::from(predicate.invoke(args, scope)?)),
            Self::Func(func) => func.invoke(args, scope),
        }
    }
}
