use std::sync::Arc;

use minijinja::value::{Enumerator, Object, ObjectRepr, ValueKind};
use minijinja::{Error, ErrorKind, State};

use crate::{Dynamic, ToValue, Value};

impl ToValue for minijinja::Value {
    fn to_value_ref(&self) -> crate::ValueRef<'_> {
        crate::ValueRef::Undefined
    }

    fn to_value(&self) -> Value {
        if let Some(reflected) = self.downcast_object_ref::<Value>() {
            return reflected.clone();
        }

        match self.kind() {
            ValueKind::Map => {
                let ty = crate::MapType::new(crate::Type::Any, crate::Type::Any, crate::Type::Any);
                let mut map = crate::Map::new(&ty);

                if let Ok(keys) = self.try_iter() {
                    for key in keys {
                        let item = self.get_item(&key).unwrap_or_default();
                        map.insert(key.to_value(), item.to_value());
                    }
                }

                Value::Map(map)
            }
            ValueKind::Seq | ValueKind::Iterable => {
                let mut items: Vec<Value> = Vec::new();

                if let Ok(values) = self.try_iter() {
                    for item in values {
                        items.push(item.to_value());
                    }
                }

                Value::Dynamic(Dynamic::from_sequence(Arc::new(items)))
            }
            ValueKind::None | ValueKind::Undefined => Value::Null,
            ValueKind::Bool => Value::Bool(self.is_true()),
            ValueKind::Number => {
                if let Ok(v) = u64::try_from(self.clone()) {
                    Value::Number(crate::Number::Int(crate::Int::U64(v)))
                } else if let Ok(v) = i64::try_from(self.clone()) {
                    Value::Number(crate::Number::Int(crate::Int::I64(v)))
                } else if let Ok(v) = f64::try_from(self.clone()) {
                    Value::Number(crate::Number::Float(crate::Float::F64(v)))
                } else {
                    Value::Null
                }
            }
            ValueKind::String => Value::Str(crate::Str::from(self.to_string())),
            _ => Value::Null,
        }
    }
}

pub fn value_to_minijinja(value: Value) -> minijinja::Value {
    match value {
        Value::Map(_) | Value::Dynamic(_) => minijinja::Value::from_object(value),
        Value::Bool(v) => minijinja::Value::from(v),
        Value::Number(v) => match v {
            crate::Number::Int(crate::Int::U64(n)) => minijinja::Value::from(n),
            crate::Number::Int(i) => minijinja::Value::from(i.to_i128() as i64),
            crate::Number::Float(f) => minijinja::Value::from(f.to_f64_raw()),
        },
        Value::Str(v) => minijinja::Value::from(v.to_string()),
        Value::Null => minijinja::Value::from(()),
        Value::Undefined => minijinja::Value::UNDEFINED,
    }
}

impl Object for Value {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        match self.as_ref() {
            Value::Map(_) => ObjectRepr::Map,
            Value::Dynamic(d) => Arc::new(d.clone()).repr(),
            _ => ObjectRepr::Plain,
        }
    }

    fn get_value(self: &Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        match self.as_ref() {
            Value::Map(m) => match m.get(&key.to_value())?.clone() {
                Value::Dynamic(d) => Some(minijinja::Value::from_object(d)),
                v => Some(minijinja::Value::from_serialize(v)),
            },
            Value::Dynamic(d) => Arc::new(d.clone()).get_value(key),
            _ => None,
        }
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        match self.as_ref() {
            Value::Map(m) => {
                let keys: Vec<minijinja::Value> = m.keys().map(minijinja::Value::from_serialize).collect();
                Enumerator::Values(keys)
            }
            Value::Dynamic(d) => Arc::new(d.clone()).enumerate(),
            _ => Enumerator::NonEnumerable,
        }
    }

    fn enumerator_len(self: &Arc<Self>) -> Option<usize> {
        match self.as_ref() {
            Value::Map(m) => Some(m.len()),
            Value::Dynamic(d) => Arc::new(d.clone()).enumerator_len(),
            _ => None,
        }
    }

    fn render(self: &Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }

    fn call(self: &Arc<Self>, state: &State<'_, '_>, args: &[minijinja::Value]) -> Result<minijinja::Value, Error> {
        match self.as_ref() {
            Value::Dynamic(d) => Arc::new(d.clone()).call(state, args),
            _ => Err(Error::new(ErrorKind::InvalidOperation, "object is not callable")),
        }
    }

    fn call_method(
        self: &Arc<Self>,
        _state: &State<'_, '_>,
        name: &str,
        args: &[minijinja::Value],
    ) -> Result<minijinja::Value, Error> {
        let Some(dynamic) = self.as_dynamic() else {
            return Err(Error::new(ErrorKind::UnknownMethod, format!("no method '{}'", name)));
        };

        let owned: Vec<Value> = args.iter().map(|a| a.to_value()).collect();
        let refs: Vec<crate::ValueRef> = owned.iter().map(Value::as_ref).collect();

        crate::Object::call(dynamic, name, &refs)
            .map(minijinja::Value::from_serialize)
            .map_err(|e| Error::new(ErrorKind::InvalidOperation, e))
    }
}

impl Object for Dynamic {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        if self.is_object() {
            ObjectRepr::Map
        } else if self.is_sequence() {
            ObjectRepr::Seq
        } else {
            ObjectRepr::Plain
        }
    }

    fn get_value(self: &Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        if let Some(obj) = self.as_object() {
            let name = if let Some(v) = key.as_str() {
                v.to_string()
            } else {
                key.as_usize()?.to_string()
            };

            match obj.field(&name) {
                Value::Dynamic(d) => Some(minijinja::Value::from_object(d)),
                v => Some(minijinja::Value::from_serialize(v)),
            }
        } else if let Some(seq) = self.as_sequence() {
            let i = key.as_usize()?;

            if i < seq.len() {
                match seq.index(i) {
                    Value::Dynamic(d) => Some(minijinja::Value::from_object(d)),
                    v => Some(minijinja::Value::from_serialize(v)),
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        if self.is_object() {
            let keys: Vec<minijinja::Value> = self
                .to_type()
                .to_struct()
                .map(|ty| {
                    ty.fields()
                        .iter()
                        .map(|f| minijinja::Value::from(f.name().to_string()))
                        .collect()
                })
                .unwrap_or_default();
            Enumerator::Values(keys)
        } else if self.is_sequence() {
            Enumerator::Seq(self.len())
        } else {
            Enumerator::NonEnumerable
        }
    }

    fn enumerator_len(self: &Arc<Self>) -> Option<usize> {
        if self.is_object() {
            self.to_type().to_struct().map(|t| t.len())
        } else if self.is_sequence() {
            Some(self.len())
        } else {
            None
        }
    }

    fn render(self: &Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }

    fn call(self: &Arc<Self>, _state: &State<'_, '_>, args: &[minijinja::Value]) -> Result<minijinja::Value, Error> {
        if !self.is_callable() {
            return Err(Error::new(ErrorKind::InvalidOperation, "object is not callable"));
        }

        let owned: Vec<Value> = args.iter().map(|a| a.to_value()).collect();
        let refs: Vec<crate::ValueRef> = owned.iter().map(Value::as_ref).collect();

        Dynamic::call(self.as_ref(), &refs)
            .map(minijinja::Value::from_serialize)
            .map_err(|e| Error::new(ErrorKind::InvalidOperation, e))
    }
}
