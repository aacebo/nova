use crate::Value;

impl From<&Value> for serde_json::Value {
    fn from(value: &Value) -> Self {
        serde_json::to_value(value).unwrap_or(serde_json::Value::Null)
    }
}

impl From<Value> for serde_json::Value {
    fn from(value: Value) -> Self {
        Self::from(&value)
    }
}

impl<'a> From<&crate::ValueRef<'a>> for serde_json::Value {
    fn from(value: &crate::ValueRef<'a>) -> Self {
        serde_json::to_value(value).unwrap_or(serde_json::Value::Null)
    }
}

impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(v) = n.as_i64() {
                    Value::Number(crate::Number::Int(crate::Int::I64(v)))
                } else if let Some(v) = n.as_u64() {
                    Value::Number(crate::Number::Int(crate::Int::U64(v)))
                } else if let Some(v) = n.as_f64() {
                    Value::Number(crate::Number::Float(crate::Float::F64(v)))
                } else {
                    Value::Null
                }
            }
            serde_json::Value::String(s) => Value::Str(crate::Str::from(s)),
            serde_json::Value::Array(items) => {
                let ty = crate::MapType::new(crate::Type::Any, crate::Type::Any, crate::Type::Any);
                let mut map = crate::Map::new(&ty);

                for (i, item) in items.into_iter().enumerate() {
                    map.insert(
                        Value::Number(crate::Number::Int(crate::Int::U64(i as u64))),
                        Value::from(item),
                    );
                }

                Value::Map(map)
            }
            serde_json::Value::Object(obj) => {
                let ty = crate::MapType::new(crate::Type::Any, crate::Type::Any, crate::Type::Any);
                let mut map = crate::Map::new(&ty);

                for (key, item) in obj {
                    map.insert(Value::Str(crate::Str::from(key)), Value::from(item));
                }

                Value::Map(map)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Value;

    #[test]
    fn json_scalars_round_trip() {
        for json in [
            serde_json::json!(true),
            serde_json::json!(42),
            serde_json::json!(-7),
            serde_json::json!("hello"),
            serde_json::json!(null),
        ] {
            let value = Value::from(json.clone());
            let back: serde_json::Value = (&value).into();
            assert_eq!(back, json);
        }
    }

    #[test]
    fn json_object_round_trips() {
        let json = serde_json::json!({
            "name": "alex",
            "age": 30,
            "admin": true,
        });

        let value = Value::from(json.clone());
        assert!(value.is_map());

        let back: serde_json::Value = value.into();
        assert_eq!(back, json);
    }

    #[test]
    fn json_array_becomes_index_keyed_map() {
        let json = serde_json::json!(["a", "b", "c"]);
        let value = Value::from(json);

        assert!(value.is_map());
        assert_eq!(value.len(), 3);

        let map = value.to_map().unwrap();
        let first = map.get(&Value::Number(crate::Number::Int(crate::Int::U64(0)))).unwrap();
        let expected: Value = "a".to_string().into();
        assert_eq!(first, &expected);
    }
}
