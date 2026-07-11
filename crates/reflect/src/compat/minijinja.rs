use std::sync::Arc;

use minijinja::value::{Enumerator, Object, ObjectRepr, ValueKind};
use minijinja::{Error, ErrorKind, State};

use crate::{ToValue, Value};

impl ToValue for minijinja::Value {
    fn to_value(&self) -> Value<'_> {
        match self.kind() {
            ValueKind::None | ValueKind::Undefined => Value::Null,
            ValueKind::Bool => Value::Bool(self.is_true()),
            ValueKind::Number => {
                if let Some(v) = self.as_i64() {
                    Value::Number(crate::Number::Int(crate::Int::I64(v)))
                } else if let Some(v) = self.as_usize() {
                    Value::Number(crate::Number::Int(crate::Int::U64(v as u64)))
                } else {
                    self.to_string()
                        .parse::<f64>()
                        .map(|v| Value::Number(crate::Number::Float(crate::Float::F64(v))))
                        .unwrap_or(Value::Null)
                }
            }
            ValueKind::String => Value::Str(crate::Str(std::borrow::Cow::Owned(self.to_string()))),
            _ => Value::Null,
        }
    }
}

impl Object for Value<'static> {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        match self.as_ref() {
            Value::Map(_) => ObjectRepr::Map,
            Value::Dynamic(d) => Arc::new(d.clone()).repr(),
            _ => ObjectRepr::Plain,
        }
    }

    fn get_value(self: &Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        match self.as_ref() {
            Value::Map(m) => match m.get(&key.to_value().into_owned())?.clone() {
                Value::Dynamic(d) => Some(minijinja::Value::from_object(d)),
                v => Some(minijinja::Value::from_serialize(v)),
            },
            Value::Dynamic(d) => Arc::new(d.clone()).get_value(key),
            Value::Ref(r) => Arc::new(r.value().clone()).get_value(key),
            Value::Mut(m) => Arc::new(m.value().clone()).get_value(key),
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

        let args: Vec<Value> = args.iter().map(|a| a.to_value()).collect();

        crate::Object::call(dynamic, name, &args)
            .map(minijinja::Value::from_serialize)
            .map_err(|e| Error::new(ErrorKind::InvalidOperation, e))
    }
}

impl Object for crate::Dynamic<'static> {
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
        write!(f, "{}", self.as_ref())
    }

    fn call(self: &Arc<Self>, _state: &State<'_, '_>, args: &[minijinja::Value]) -> Result<minijinja::Value, Error> {
        if !self.is_callable() {
            return Err(Error::new(ErrorKind::InvalidOperation, "object is not callable"));
        }

        let args: Vec<Value> = args.iter().map(|a| a.to_value()).collect();

        crate::Dynamic::call(self.as_ref(), &args)
            .map(minijinja::Value::from_serialize)
            .map_err(|e| Error::new(ErrorKind::InvalidOperation, e))
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use minijinja::Environment;

    use crate::{Number, Value};

    fn user_map() -> Value<'static> {
        let ty = crate::MapType::new(crate::Type::Any, crate::Type::Any, crate::Type::Any);
        let mut map = crate::Map::new(&ty);
        map.insert(Value::Str(crate::Str("name".into())), Value::Str(crate::Str("alex".into())));
        map.insert(
            Value::Str(crate::Str("age".into())),
            Value::Number(Number::Int(crate::Int::U64(30))),
        );
        Value::Map(map)
    }

    #[test]
    fn map_get_value_reads_fields() {
        let value = minijinja::Value::from_object(user_map());
        let env = Environment::new();
        let out = env
            .render_str("{{ v.name }}-{{ v.age }}", minijinja::context! { v => value })
            .unwrap();
        assert_eq!(out, "alex-30");
    }

    #[test]
    fn map_enumerates_keys() {
        let value = minijinja::Value::from_object(user_map());
        let env = Environment::new();
        let out = env
            .render_str("{% for k in v %}{{ k }},{% endfor %}", minijinja::context! { v => value })
            .unwrap();
        assert!(out.contains("name"));
        assert!(out.contains("age"));
    }

    #[test]
    fn repr_reports_map() {
        let obj: Arc<Value<'static>> = Arc::new(user_map());
        assert!(matches!(
            minijinja::value::Object::repr(&obj),
            minijinja::value::ObjectRepr::Map
        ));
    }
}
