use std::collections::BTreeMap;

use reflect::ToValue;

use crate::{Error, Schema, Validate, error};

pub fn object() -> ObjectSchema {
    ObjectSchema::default()
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObjectSchema {
    fields: BTreeMap<String, Schema>,
}

impl ObjectSchema {
    pub fn field(mut self, name: impl std::fmt::Display, schema: impl Into<Schema>) -> Self {
        self.fields.insert(name.to_string(), schema.into());
        self
    }
}

impl Validate for ObjectSchema {
    fn validate(&self, value: &reflect::Value) -> Result<(), Error> {
        let map = value
            .as_map()
            .ok_or(("type", format!("expected object, received {}", value.to_type())))?;
        let mut errors = error::object();

        for (name, schema) in &self.fields {
            let key = name.clone();
            let field = map.get(&key.to_value()).cloned().unwrap_or(reflect::Value::Undefined);

            if let Err(err) = schema.validate(&field) {
                errors = errors.field(name, err);
            }
        }

        errors.ok()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use reflect::{ToValue, btree_map};

    use crate::{Error, Validate, object, string};

    #[test]
    fn fail() {
        let schema = object();
        let value = "test".to_value();
        let res = schema.validate(&value);
        assert!(res.is_err());
    }

    #[test]
    fn succeed() -> Result<(), Error> {
        let schema = object();
        let map = btree_map! { "name".to_string() => "nova".to_string() };
        let value = map.to_value();
        schema.validate(&value)?;
        Ok(())
    }

    mod field {
        use super::*;

        #[test]
        fn fail() {
            let schema = object().field("name", string());
            let map = btree_map! { "name".to_string() => 123_i32 };
            let value = map.to_value();
            let res = schema.validate(&value);
            assert!(res.is_err());
        }

        #[test]
        fn succeed() -> Result<(), Error> {
            let schema = object().field("name", string());
            let map = btree_map! { "name".to_string() => "nova".to_string() };
            let value = map.to_value();
            schema.validate(&value)?;
            Ok(())
        }
    }

    mod serde {
        use super::*;

        #[test]
        fn round_trip() -> Result<(), Error> {
            let schema = object().field("name", string().min(1));
            let json = serde_json::to_string(&schema).unwrap();
            let schema: crate::ObjectSchema = serde_json::from_str(&json).unwrap();

            let ok = btree_map! { "name".to_string() => "nova".to_string() };
            schema.validate(&ok.to_value())?;

            let bad = btree_map! { "name".to_string() => "".to_string() };
            assert!(schema.validate(&bad.to_value()).is_err());
            Ok(())
        }
    }
}
