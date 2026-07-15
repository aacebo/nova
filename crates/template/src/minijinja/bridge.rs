use std::sync::Arc;

use minijinja::value::{Enumerator, Object, ObjectRepr, ValueKind};
use minijinja::{Error, ErrorKind, State};
use nova_reflect::{Int, Number, Value, ValueRef};

use crate::{Args, Context, KArgs, Pointer};

const CONTEXT_KEY: &str = "__$ctx__";

pub(crate) fn from_minijinja(value: &minijinja::Value) -> Pointer {
    if let Some(pointer) = value.downcast_object_ref::<Pointer>() {
        return pointer.clone();
    }

    match value.kind() {
        ValueKind::Map => {
            let mut entries: std::collections::BTreeMap<Pointer, Pointer> = std::collections::BTreeMap::new();

            if let Ok(keys) = value.try_iter() {
                for key in keys {
                    let item = value.get_item(&key).unwrap_or_default();
                    entries.insert(from_minijinja(&key), from_minijinja(&item));
                }
            }

            Pointer::new(entries)
        }
        ValueKind::Seq | ValueKind::Iterable => {
            let mut items: Vec<Pointer> = Vec::new();

            if let Ok(values) = value.try_iter() {
                for item in values {
                    items.push(from_minijinja(&item));
                }
            }

            Pointer::new(items)
        }
        _ => Pointer::new(scalar_from_minijinja(value)),
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

pub(crate) fn to_minijinja(value: ValueRef<'_>) -> minijinja::Value {
    match value {
        ValueRef::Bool(v) => minijinja::Value::from(v),
        ValueRef::Number(v) => match v {
            Number::Int(Int::U64(n)) => minijinja::Value::from(n),
            Number::Int(i) => minijinja::Value::from(i.to_i128() as i64),
            Number::Float(f) => minijinja::Value::from(f.to_f64_raw()),
        },
        ValueRef::Str(v) => minijinja::Value::from(v.to_string()),
        ValueRef::Null => minijinja::Value::from(()),
        ValueRef::Undefined => minijinja::Value::UNDEFINED,
        ValueRef::Dynamic(_) => minijinja::Value::UNDEFINED,
        other => minijinja::Value::from_object(Pointer::new(other.to_owned())),
    }
}

pub(crate) fn pointer_to_minijinja(pointer: Pointer) -> minijinja::Value {
    let composite = {
        let value = pointer.value();
        value.is_dynamic() || value.is_map()
    };

    if composite || pointer.is_callable() || pointer.as_namespace().is_some() {
        return minijinja::Value::from_object(pointer);
    }

    to_minijinja(pointer.value())
}

fn kwargs_to_kargs(kwargs: minijinja::value::Kwargs) -> Result<KArgs, Error> {
    let mut kargs = KArgs::new();

    for key in kwargs.args() {
        let value: minijinja::Value = kwargs.get(key)?;
        kargs.set(key, from_minijinja(&value));
    }

    kwargs.assert_all_used()?;
    Ok(kargs)
}

fn to_args(state: &State<'_, '_>, args: &[minijinja::Value]) -> Result<Args, Error> {
    let (positional, kwargs): (&[minijinja::Value], minijinja::value::Kwargs) = minijinja::value::from_args(args)?;

    let args = Args::new(
        positional.iter().map(from_minijinja).collect::<Vec<_>>(),
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
            ValueRef::Map(_) => ObjectRepr::Map,
            ValueRef::Dynamic(d) if d.is_sequence() => ObjectRepr::Seq,
            ValueRef::Dynamic(d) if d.is_object() => ObjectRepr::Map,
            _ => ObjectRepr::Plain,
        }
    }

    fn get_value(self: &Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        if self.as_namespace().is_some() {
            return self.field(key.as_str()?).map(pointer_to_minijinja);
        }

        let value = self.value();

        if value.is_map() {
            return self.key(scalar_from_minijinja(key)).map(pointer_to_minijinja);
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

        if let ValueRef::Map(map) = &value {
            let keys: Vec<minijinja::Value> = map.keys().map(|k| to_minijinja(k.as_ref())).collect();
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
            ValueRef::Map(map) => Some(map.len()),
            ValueRef::Dynamic(d) if d.is_sequence() => Some(d.len()),
            ValueRef::Dynamic(d) if d.is_object() => d.to_type().to_struct().map(|t| t.len()),
            _ => None,
        }
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

        let value = self.value();
        let object = value
            .as_dynamic()
            .and_then(|d| d.as_object())
            .ok_or_else(|| Error::new(ErrorKind::UnknownMethod, format!("no method '{name}'")))?;

        let owned: Vec<Pointer> = args.iter().map(from_minijinja).collect();
        let values: Vec<ValueRef> = owned.iter().map(|p| p.value()).collect();

        nova_reflect::Object::call(object, name, &values)
            .map(|v| to_minijinja(v.as_ref()))
            .map_err(|e| Error::new(ErrorKind::InvalidOperation, e))
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
