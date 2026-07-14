use std::sync::Arc;

use crate::{Action, Args, Call, Pointer, Predicate, Scope, ToType, ToValue, Type, Value};

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

    pub fn invoke(&self, args: &Args, scope: &Scope) -> Result<Pointer, Box<dyn std::error::Error>> {
        self.callback.invoke(args, scope)
    }
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function").field("name", &self.name).finish_non_exhaustive()
    }
}

impl ToType for Function {
    fn to_type(&self) -> Type {
        Type::Any
    }
}

impl ToValue for Function {
    fn to_value(&self) -> Value<'_> {
        Value::Undefined
    }
}

impl nova_template::Call for Function {
    fn call(&self, args: &Args) -> Result<Pointer, nova_template::Error> {
        let scope = args
            .caller()
            .and_then(|c| c.downcast::<Scope>())
            .ok_or_else(|| nova_template::Error::message("no scope bound to template render"))?;

        let child = scope.fork(&self.name, args.args().to_vec(), args.kargs().clone());
        let child_args = Args::new(child.args().to_vec(), child.kargs().clone());

        self.callback
            .invoke(&child_args, &child)
            .map_err(|err| nova_template::Error::message(err.to_string()))
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

    pub fn invoke(&self, args: &Args, scope: &Scope) -> Result<Pointer, Box<dyn std::error::Error>> {
        match self {
            Self::Action(action) => {
                action.invoke(args, scope)?;
                Ok(Pointer::new(Value::Null))
            }
            Self::Predicate(predicate) => Ok(Pointer::new(Value::Bool(predicate.invoke(args, scope)?))),
            Self::Func(func) => func.invoke(args, scope),
        }
    }
}
