use std::sync::Arc;

use minijinja::value::Kwargs;

use crate::{Action, Args, Call, Context, Predicate, Value};

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

    pub fn invoke(&self, ctx: &mut Context) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        match self {
            Self::Action(action) => {
                action.invoke(ctx)?;
                Ok(None)
            }
            Self::Predicate(predicate) => Ok(Some(Value::from(predicate.invoke(ctx)?))),
            Self::Func(func) => func.invoke(ctx),
        }
    }
}

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

    pub fn invoke(&self, ctx: &mut Context) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        self.callback.invoke(ctx)
    }
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function").field("name", &self.name).finish_non_exhaustive()
    }
}

impl minijinja::value::Object for Function {
    fn call(self: &Arc<Self>, state: &minijinja::State<'_, '_>, args: &[Value]) -> Result<Value, minijinja::Error> {
        let (positional, kwargs): (&[Value], Kwargs) = minijinja::value::from_args(args)?;

        if !positional.is_empty() {
            return Err(minijinja::Error::new(
                minijinja::ErrorKind::InvalidOperation,
                format!("\"{}\" takes keyword arguments only", self.name),
            ));
        }

        let ctx = state
            .lookup(Context::KEY)
            .and_then(|v| v.downcast_object::<Context>())
            .ok_or_else(|| {
                minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, "no context bound to template render")
            })?;

        let args = Args::from_kwargs(kwargs)?;
        let mut child = ctx.child(args);
        let value = self
            .callback
            .invoke(&mut child)
            .map_err(|err| minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, err.to_string()))?;
        ctx.scope().merge(&self.name, child.scope());
        Ok(value.unwrap_or(Value::UNDEFINED))
    }
}
