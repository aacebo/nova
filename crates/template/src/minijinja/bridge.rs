use std::sync::Arc;

use minijinja::value::{Enumerator, Object, ObjectRepr, ValueKind};
use minijinja::{Error, ErrorKind, State};
use nova_reflect::{Int, Number, Value};

use crate::{Args, Context, KArgs, Pointer};

const CONTEXT_KEY: &str = "__$ctx__";

pub(crate) fn from_minijinja(value: &minijinja::Value) -> Value<'static> {
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
        ValueKind::String => Value::Str(nova_reflect::Str(std::borrow::Cow::Owned(value.to_string()))),
        ValueKind::Map => {
            let ty = nova_reflect::MapType::new(nova_reflect::Type::Any, nova_reflect::Type::Any, nova_reflect::Type::Any);
            let mut map = nova_reflect::Map::new(&ty);

            if let Ok(keys) = value.try_iter() {
                for key in keys {
                    let item = value.get_item(&key).unwrap_or_default();
                    map.insert(from_minijinja(&key), from_minijinja(&item));
                }
            }

            Value::Map(map)
        }
        ValueKind::Seq | ValueKind::Iterable => {
            let ty = nova_reflect::MapType::new(nova_reflect::Type::Any, nova_reflect::Type::Any, nova_reflect::Type::Any);
            let mut map = nova_reflect::Map::new(&ty);

            if let Ok(items) = value.try_iter() {
                for (i, item) in items.enumerate() {
                    map.insert(Value::Number(Number::Int(Int::U64(i as u64))), from_minijinja(&item));
                }
            }

            Value::Map(map)
        }
        _ => Value::Null,
    }
}

pub(crate) fn to_minijinja(value: Value<'_>) -> minijinja::Value {
    match value {
        Value::Bool(v) => minijinja::Value::from(v),
        Value::Number(v) => match v {
            Number::Int(Int::U64(n)) => minijinja::Value::from(n),
            Number::Int(i) => minijinja::Value::from(i.to_i128() as i64),
            Number::Float(f) => minijinja::Value::from(f.to_f64_raw()),
        },
        Value::Str(v) => minijinja::Value::from(v.to_string()),
        Value::Null => minijinja::Value::from(()),
        Value::Undefined => minijinja::Value::UNDEFINED,
        Value::Ref(v) => to_minijinja(v.value.clone()),
        Value::Mut(v) => to_minijinja(v.value.clone()),
        Value::Dynamic(_) => minijinja::Value::UNDEFINED,
        other => minijinja::Value::from_object(Pointer::new(other.into_owned())),
    }
}

pub(crate) fn pointer_to_minijinja(pointer: Pointer) -> minijinja::Value {
    if pointer.is_callable() || pointer.as_namespace().is_some() || pointer.value().is_dynamic() {
        return minijinja::Value::from_object(pointer);
    }

    to_minijinja(pointer.value())
}

fn kwargs_to_kargs(kwargs: minijinja::value::Kwargs) -> Result<KArgs, Error> {
    let mut kargs = KArgs::new();

    for key in kwargs.args() {
        let value: minijinja::Value = kwargs.get(key)?;
        kargs.set(key, Pointer::new(from_minijinja(&value)));
    }

    kwargs.assert_all_used()?;
    Ok(kargs)
}

fn to_args(state: &State<'_, '_>, args: &[minijinja::Value]) -> Result<Args, Error> {
    let (positional, kwargs): (&[minijinja::Value], minijinja::value::Kwargs) = minijinja::value::from_args(args)?;

    let args = Args::new(
        positional.iter().map(|v| Pointer::new(from_minijinja(v))).collect::<Vec<_>>(),
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

        match self.value() {
            Value::Map(_) => ObjectRepr::Map,
            Value::Dynamic(d) if d.is_sequence() => ObjectRepr::Seq,
            Value::Dynamic(d) if d.is_object() => ObjectRepr::Map,
            _ => ObjectRepr::Plain,
        }
    }

    fn get_value(self: &Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        if self.as_namespace().is_some() {
            return self.field(key.as_str()?).map(pointer_to_minijinja);
        }

        let value = self.value();

        if let Value::Map(map) = &value {
            return map.get(&from_minijinja(key)).map(|v| to_minijinja(v.clone()));
        }

        let dynamic = value.as_dynamic()?;

        if dynamic.is_object() {
            let name = match key.as_str() {
                Some(v) => v.to_string(),
                None => key.as_usize()?.to_string(),
            };

            return self.field(&name).map(pointer_to_minijinja);
        }

        if dynamic.is_sequence() {
            return self.index(key.as_usize()?).map(pointer_to_minijinja);
        }

        None
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        let value = self.value();

        if let Value::Map(map) = &value {
            let keys: Vec<minijinja::Value> = map.keys().map(|k| to_minijinja(k.clone())).collect();
            return Enumerator::Values(keys);
        }

        let Some(dynamic) = value.as_dynamic() else {
            return Enumerator::NonEnumerable;
        };

        if dynamic.is_sequence() {
            return Enumerator::Seq(dynamic.len());
        }

        if dynamic.is_object() {
            let keys: Vec<minijinja::Value> = dynamic
                .to_type()
                .to_struct()
                .map(|ty| {
                    ty.fields()
                        .iter()
                        .map(|f| minijinja::Value::from(f.name().to_string()))
                        .collect()
                })
                .unwrap_or_default();

            return Enumerator::Values(keys);
        }

        Enumerator::NonEnumerable
    }

    fn enumerator_len(self: &Arc<Self>) -> Option<usize> {
        let value = self.value();

        match &value {
            Value::Map(map) => Some(map.len()),
            Value::Dynamic(d) if d.is_sequence() => Some(d.len()),
            Value::Dynamic(d) if d.is_object() => d.to_type().to_struct().map(|t| t.len()),
            _ => None,
        }
    }

    fn render(self: &Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }

    fn call(self: &Arc<Self>, state: &State<'_, '_>, args: &[minijinja::Value]) -> Result<minijinja::Value, Error> {
        if let Some(call) = self.as_call() {
            let args = to_args(state, args)?;

            return call
                .call(&args)
                .map(pointer_to_minijinja)
                .map_err(|e| Error::new(ErrorKind::InvalidOperation, e.to_string()));
        }

        let value = self.value();
        let callable = value
            .as_dynamic()
            .and_then(|d| d.as_callable())
            .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "object is not callable"))?;

        let values: Vec<Value> = args.iter().map(from_minijinja).collect();

        nova_reflect::Callable::call(callable, &values)
            .map(to_minijinja)
            .map_err(|e| Error::new(ErrorKind::InvalidOperation, e))
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

        let value = self.value();
        let object = value
            .as_dynamic()
            .and_then(|d| d.as_object())
            .ok_or_else(|| Error::new(ErrorKind::UnknownMethod, format!("no method '{name}'")))?;

        let values: Vec<Value> = args.iter().map(from_minijinja).collect();

        nova_reflect::Object::call(object, name, &values)
            .map(to_minijinja)
            .map_err(|e| Error::new(ErrorKind::InvalidOperation, e))
    }
}

#[derive(Debug)]
pub(crate) struct ContextObject(Arc<dyn Context>);

impl ContextObject {
    pub(crate) fn new(ctx: Arc<dyn Context>) -> Self {
        Self(ctx)
    }

    fn caller(&self) -> Pointer {
        self.0.as_caller()
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
