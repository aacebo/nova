use std::sync::Arc;

use minijinja::value::{Enumerator, Object, ObjectRepr};
use minijinja::{ErrorKind, State};
use nova_core::{Args, Binding, Context, Error, KArgs, TemplateEngine};
use nova_reflect::ToValue;
use nova_reflect::compat::minijinja::value_to_minijinja;

use crate::Scope;

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

    fn root(ctx: &Scope) -> minijinja::Value {
        minijinja::Value::from_object(ContextObject(ctx.clone()))
    }
}

impl std::fmt::Debug for Minijinja {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Minijinja").finish_non_exhaustive()
    }
}

impl TemplateEngine for Minijinja {
    type Context = Scope;

    fn render(&self, src: &str, ctx: &Self::Context) -> Result<String, Error> {
        self.env.render_str(src, Self::root(ctx)).map_err(into_error)
    }

    fn eval(&self, src: &str, ctx: &Self::Context) -> Result<nova_reflect::Value, Error> {
        let expr = self.env.compile_expression(src).map_err(into_error)?;
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

pub fn from_minijinja(value: &minijinja::Value) -> nova_reflect::Value {
    if let Some(binding) = value.downcast_object_ref::<BindingObject>() {
        return binding.binding.to_dynamic_value();
    }

    if let Some(reflected) = value.downcast_object_ref::<nova_reflect::Value>() {
        return reflected.clone();
    }

    value.to_value()
}

pub fn binding_to_minijinja(name: &str, binding: Binding) -> minijinja::Value {
    if binding.is_callable() || binding.as_namespace().is_some() {
        return minijinja::Value::from_object(BindingObject {
            name: name.to_string(),
            binding,
        });
    }

    value_to_minijinja(binding.into_value())
}

fn kwargs_to_kargs(kwargs: minijinja::value::Kwargs) -> Result<KArgs, minijinja::Error> {
    let mut kargs = KArgs::new();

    for key in kwargs.args() {
        let value: minijinja::Value = kwargs.get(key)?;
        kargs.set(key, from_minijinja(&value));
    }

    kwargs.assert_all_used()?;
    Ok(kargs)
}

fn to_args(args: &[minijinja::Value]) -> Result<Args, minijinja::Error> {
    let (positional, kwargs): (&[minijinja::Value], minijinja::value::Kwargs) = minijinja::value::from_args(args)?;

    Ok(Args::new(
        positional.iter().map(from_minijinja).collect::<Vec<_>>(),
        kwargs_to_kargs(kwargs)?,
    ))
}

fn state_scope(state: &State<'_, '_>) -> Result<Scope, minijinja::Error> {
    state
        .lookup(CONTEXT_KEY)
        .and_then(|v| v.downcast_object::<ContextObject>())
        .map(|ctx| ctx.0.clone())
        .ok_or_else(|| minijinja::Error::new(ErrorKind::InvalidOperation, "no scope bound to template render"))
}

#[derive(Debug)]
struct BindingObject {
    name: String,
    binding: Binding,
}

impl BindingObject {
    fn invoke(&self, state: &State<'_, '_>, args: &[minijinja::Value]) -> Result<minijinja::Value, minijinja::Error> {
        let call = self
            .binding
            .as_call()
            .ok_or_else(|| minijinja::Error::new(ErrorKind::InvalidOperation, "object is not callable"))?;

        let args = to_args(args)?;
        let scope = state_scope(state)?;
        let next = scope.next(&self.name, args);

        call.call(&next)
            .map(|out| binding_to_minijinja(&self.name, out))
            .map_err(|e| minijinja::Error::new(ErrorKind::InvalidOperation, e.to_string()))
    }
}

impl Object for BindingObject {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        if self.binding.as_namespace().is_some() {
            return ObjectRepr::Map;
        }

        ObjectRepr::Plain
    }

    fn get_value(self: &Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        if self.binding.as_namespace().is_some() {
            let name = key.as_str()?;
            return self.binding.field(name).map(|member| binding_to_minijinja(name, member));
        }

        None
    }

    fn render(self: &Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.binding.value())
    }

    fn call(self: &Arc<Self>, state: &State<'_, '_>, args: &[minijinja::Value]) -> Result<minijinja::Value, minijinja::Error> {
        self.invoke(state, args)
    }

    fn call_method(
        self: &Arc<Self>,
        state: &State<'_, '_>,
        name: &str,
        args: &[minijinja::Value],
    ) -> Result<minijinja::Value, minijinja::Error> {
        if let Some(member) = self.binding.field(name)
            && member.is_callable()
        {
            let member = BindingObject {
                name: name.to_string(),
                binding: member,
            };

            return member.invoke(state, args);
        }

        Err(minijinja::Error::new(ErrorKind::UnknownMethod, format!("no method '{name}'")))
    }
}

#[derive(Debug)]
struct ContextObject(Scope);

impl Object for ContextObject {
    fn get_value(self: &Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        let name = key.as_str()?;

        if name == CONTEXT_KEY {
            return Some(minijinja::Value::from_dyn_object(self.clone()));
        }

        Context::get(&self.0, name).map(|binding| binding_to_minijinja(name, binding))
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Values(self.0.iter().map(|(key, _)| value_to_minijinja(key)).collect())
    }
}
