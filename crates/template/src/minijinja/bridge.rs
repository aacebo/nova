use std::sync::Arc;

use minijinja::value::{Enumerator, Object, ObjectRepr, ValueKind};
use minijinja::{Error, ErrorKind, State};
use nova_reflect::{Int, Number, Value};

use crate::{Args, Context, KArgs, Pointer};

const CONTEXT_KEY: &str = "__$ctx__";

pub(crate) fn from_minijinja(value: &minijinja::Value) -> Pointer {
    if let Some(pointer) = value.downcast_object_ref::<Pointer>() {
        return pointer.clone();
    }

    if let Some(reflected) = value.downcast_object_ref::<Value>() {
        return Pointer::Value(reflected.clone());
    }

    Pointer::Value(value_from_minijinja(value))
}

fn value_from_minijinja(value: &minijinja::Value) -> Value {
    if let Some(reflected) = value.downcast_object_ref::<Value>() {
        return reflected.clone();
    }

    match value.kind() {
        ValueKind::Map => {
            let ty = nova_reflect::MapType::new(nova_reflect::Type::Any, nova_reflect::Type::Any, nova_reflect::Type::Any);
            let mut map = nova_reflect::Map::new(&ty);

            if let Ok(keys) = value.try_iter() {
                for key in keys {
                    let item = value.get_item(&key).unwrap_or_default();
                    map.insert(value_from_minijinja(&key), value_from_minijinja(&item));
                }
            }

            Value::Map(map)
        }
        ValueKind::Seq | ValueKind::Iterable => {
            let mut items: Vec<Value> = Vec::new();

            if let Ok(values) = value.try_iter() {
                for item in values {
                    items.push(value_from_minijinja(&item));
                }
            }

            nova_reflect::value_of!(items)
        }
        _ => scalar_from_minijinja(value),
    }
}

fn scalar_from_minijinja(value: &minijinja::Value) -> Value {
    match value.kind() {
        ValueKind::None | ValueKind::Undefined => Value::Null,
        ValueKind::Bool => Value::Bool(value.is_true()),
        ValueKind::Number => {
            if let Ok(v) = u64::try_from(value.clone()) {
                Value::Number(Number::Int(Int::U64(v)))
            } else if let Ok(v) = i64::try_from(value.clone()) {
                Value::Number(Number::Int(Int::I64(v)))
            } else if let Ok(v) = f64::try_from(value.clone()) {
                Value::Number(Number::Float(nova_reflect::Float::F64(v)))
            } else {
                Value::Null
            }
        }
        ValueKind::String => Value::Str(nova_reflect::Str::from(value.to_string())),
        _ => Value::Null,
    }
}

pub(crate) fn value_to_minijinja(value: Value) -> minijinja::Value {
    match value {
        Value::Map(_) | Value::Dynamic(_) => minijinja::Value::from_object(value),
        Value::Bool(v) => minijinja::Value::from(v),
        Value::Number(v) => match v {
            Number::Int(Int::U64(n)) => minijinja::Value::from(n),
            Number::Int(i) => minijinja::Value::from(i.to_i128() as i64),
            Number::Float(f) => minijinja::Value::from(f.to_f64_raw()),
        },
        Value::Str(v) => minijinja::Value::from(v.to_string()),
        Value::Null => minijinja::Value::from(()),
        Value::Undefined => minijinja::Value::UNDEFINED,
    }
}

pub(crate) fn pointer_to_minijinja(pointer: Pointer) -> minijinja::Value {
    if pointer.is_callable() || pointer.as_namespace().is_some() {
        return minijinja::Value::from_object(pointer);
    }

    value_to_minijinja(pointer.into_value())
}

fn kwargs_to_kargs(kwargs: minijinja::value::Kwargs) -> Result<KArgs, Error> {
    let mut kargs = KArgs::new();

    for key in kwargs.args() {
        let value: minijinja::Value = kwargs.get(key)?;
        kargs.set(key, value_from_minijinja(&value));
    }

    kwargs.assert_all_used()?;
    Ok(kargs)
}

fn to_args(state: &State<'_, '_>, args: &[minijinja::Value]) -> Result<Args, Error> {
    let (positional, kwargs): (&[minijinja::Value], minijinja::value::Kwargs) = minijinja::value::from_args(args)?;

    let args = Args::new(
        positional.iter().map(value_from_minijinja).collect::<Vec<_>>(),
        kwargs_to_kargs(kwargs)?,
    );

    Ok(
        match state.lookup(CONTEXT_KEY).and_then(|v| v.downcast_object::<ContextObject>()) {
            Some(ctx) => args.with_caller(ctx.caller()),
            None => args,
        },
    )
}

impl Object for Pointer {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        if self.as_namespace().is_some() {
            return ObjectRepr::Map;
        }

        ObjectRepr::Plain
    }

    fn get_value(self: &Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        if self.as_namespace().is_some() {
            return self.field(key.as_str()?).map(pointer_to_minijinja);
        }

        None
    }

    fn render(self: &Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }

    fn call(self: &Arc<Self>, state: &State<'_, '_>, args: &[minijinja::Value]) -> Result<minijinja::Value, Error> {
        let call = self
            .as_call()
            .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "object is not callable"))?;

        let args = to_args(state, args)?;

        call.call(&args)
            .map(pointer_to_minijinja)
            .map_err(|e| Error::new(ErrorKind::InvalidOperation, e.to_string()))
    }

    fn call_method(
        self: &Arc<Self>,
        state: &State<'_, '_>,
        name: &str,
        args: &[minijinja::Value],
    ) -> Result<minijinja::Value, Error> {
        if let Some(member) = self.field(name)
            && let Some(call) = member.as_call()
        {
            let args = to_args(state, args)?;

            return call
                .call(&args)
                .map(pointer_to_minijinja)
                .map_err(|e| Error::new(ErrorKind::InvalidOperation, e.to_string()));
        }

        Err(Error::new(ErrorKind::UnknownMethod, format!("no method '{name}'")))
    }
}

#[derive(Debug)]
pub(crate) struct ContextObject(Arc<dyn Context>);

impl ContextObject {
    pub(crate) fn new(ctx: Arc<dyn Context>) -> Self {
        Self(ctx)
    }

    fn caller(&self) -> Arc<dyn Context> {
        self.0.clone()
    }
}

impl Object for ContextObject {
    fn get_value(self: &Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        let name = key.as_str()?;

        if name == CONTEXT_KEY {
            return Some(minijinja::Value::from_dyn_object(self.clone()));
        }

        self.0.resolve(name).map(pointer_to_minijinja)
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Values(self.0.names().into_iter().map(minijinja::Value::from).collect())
    }
}
