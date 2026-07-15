use std::sync::Arc;

use minijinja::value::{Enumerator, Object, ObjectRepr};
use minijinja::{ErrorKind, State};
use nova_core::{Args, Binding, Context, Engine, Error, KArgs};
use nova_reflect::ToValue;
use nova_reflect::compat::minijinja::value_to_minijinja;

const CONTEXT_KEY: &str = "__$ctx__";

pub struct Minijinja {
    env: minijinja::Environment<'static>,
}

impl Default for Minijinja {
    fn default() -> Self {
        Self::new()
    }
}

impl Minijinja {
    pub fn new() -> Self {
        Self {
            env: minijinja::Environment::new(),
        }
    }

    fn root(ctx: &Arc<dyn Context>) -> minijinja::Value {
        minijinja::Value::from_object(ContextObject::new(ctx.clone()))
    }
}

impl std::fmt::Debug for Minijinja {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Minijinja").finish_non_exhaustive()
    }
}

impl Engine for Minijinja {
    fn add_template(&mut self, name: &str, source: &str) -> Result<(), Error> {
        self.env
            .add_template_owned(name.to_string(), source.to_string())
            .map_err(into_error)?;
        Ok(())
    }

    fn render(&self, name: &str, ctx: &Arc<dyn Context>) -> Result<String, Error> {
        let template = self.env.get_template(name).map_err(into_error)?;
        template.render(Self::root(ctx)).map_err(into_error)
    }

    fn render_str(&self, source: &str, ctx: &Arc<dyn Context>) -> Result<String, Error> {
        self.env.render_str(source, Self::root(ctx)).map_err(into_error)
    }

    fn eval(&self, expr: &str, ctx: &Arc<dyn Context>) -> Result<Binding, Error> {
        let expr = self.env.compile_expression(expr).map_err(into_error)?;
        let value = expr.eval(Self::root(ctx)).map_err(into_error)?;
        Ok(from_minijinja(&value))
    }
}

fn into_error(value: minijinja::Error) -> Error {
    match value.line() {
        Some(line) => Error::message(format!("{value} (line {line})")),
        None => Error::message(value.to_string()),
    }
}

pub fn from_minijinja(value: &minijinja::Value) -> Binding {
    if let Some(binding) = value.downcast_object_ref::<BindingObject>() {
        return binding.0.clone();
    }

    if let Some(reflected) = value.downcast_object_ref::<nova_reflect::Value>() {
        return Binding::Value(reflected.clone());
    }

    Binding::Value(value.to_value())
}

pub fn binding_to_minijinja(binding: Binding) -> minijinja::Value {
    if binding.is_callable() || binding.as_namespace().is_some() {
        return minijinja::Value::from_object(BindingObject(binding));
    }

    value_to_minijinja(binding.into_value())
}

fn kwargs_to_kargs(kwargs: minijinja::value::Kwargs) -> Result<KArgs, minijinja::Error> {
    let mut kargs = KArgs::new();

    for key in kwargs.args() {
        let value: minijinja::Value = kwargs.get(key)?;
        kargs.set(key, value.to_value());
    }

    kwargs.assert_all_used()?;
    Ok(kargs)
}

fn to_args(args: &[minijinja::Value]) -> Result<Args, minijinja::Error> {
    let (positional, kwargs): (&[minijinja::Value], minijinja::value::Kwargs) = minijinja::value::from_args(args)?;

    Ok(Args::new(
        positional.iter().map(|v| v.to_value()).collect::<Vec<_>>(),
        kwargs_to_kargs(kwargs)?,
    ))
}

fn state_context(state: &State<'_, '_>) -> Result<Arc<dyn Context>, minijinja::Error> {
    state
        .lookup(CONTEXT_KEY)
        .and_then(|v| v.downcast_object::<ContextObject>())
        .map(|ctx| ctx.0.clone())
        .ok_or_else(|| minijinja::Error::new(ErrorKind::InvalidOperation, "no scope bound to template render"))
}

#[derive(Debug)]
struct BindingObject(Binding);

impl Object for BindingObject {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        if self.0.as_namespace().is_some() {
            return ObjectRepr::Map;
        }

        ObjectRepr::Plain
    }

    fn get_value(self: &Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        if self.0.as_namespace().is_some() {
            return self.0.field(key.as_str()?).map(binding_to_minijinja);
        }

        None
    }

    fn render(self: &Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.value())
    }

    fn call(self: &Arc<Self>, state: &State<'_, '_>, args: &[minijinja::Value]) -> Result<minijinja::Value, minijinja::Error> {
        let call = self
            .0
            .as_call()
            .ok_or_else(|| minijinja::Error::new(ErrorKind::InvalidOperation, "object is not callable"))?;

        let args = to_args(args)?;
        let ctx = state_context(state)?;

        call.call(&args, ctx.as_ref())
            .map(binding_to_minijinja)
            .map_err(|e| minijinja::Error::new(ErrorKind::InvalidOperation, e.to_string()))
    }

    fn call_method(
        self: &Arc<Self>,
        state: &State<'_, '_>,
        name: &str,
        args: &[minijinja::Value],
    ) -> Result<minijinja::Value, minijinja::Error> {
        if let Some(member) = self.0.field(name)
            && let Some(call) = member.as_call()
        {
            let args = to_args(args)?;
            let ctx = state_context(state)?;

            return call
                .call(&args, ctx.as_ref())
                .map(binding_to_minijinja)
                .map_err(|e| minijinja::Error::new(ErrorKind::InvalidOperation, e.to_string()));
        }

        Err(minijinja::Error::new(ErrorKind::UnknownMethod, format!("no method '{name}'")))
    }
}

#[derive(Debug)]
struct ContextObject(Arc<dyn Context>);

impl ContextObject {
    fn new(ctx: Arc<dyn Context>) -> Self {
        Self(ctx)
    }
}

impl Object for ContextObject {
    fn get_value(self: &Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        let name = key.as_str()?;

        if name == CONTEXT_KEY {
            return Some(minijinja::Value::from_dyn_object(self.clone()));
        }

        self.0.resolve(name).map(binding_to_minijinja)
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Values(self.0.names().into_iter().map(minijinja::Value::from).collect())
    }
}
