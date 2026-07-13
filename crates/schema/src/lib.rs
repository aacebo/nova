mod array;
mod bool;
pub mod error;
mod integer;
mod null;
mod number;
mod object;
mod oneof;
mod string;

pub use array::*;
pub use bool::*;
pub use error::Error;
pub use integer::*;
pub use null::*;
pub use number::*;
pub use object::*;
pub use oneof::*;
pub use string::*;

pub trait Validate {
    fn validate(&self, value: &reflect::Value) -> Result<(), Error>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Schema {
    #[serde(flatten)]
    inner: AnySchema,
    message: Option<String>,
    optional: Option<bool>,
}

impl Schema {
    pub fn message(mut self, message: impl std::fmt::Display) -> Self {
        self.message = Some(message.to_string());
        self
    }

    pub fn required(mut self) -> Self {
        self.optional = Some(false);
        self
    }

    pub fn optional(mut self) -> Self {
        self.optional = Some(true);
        self
    }
}

impl<T: Into<AnySchema> + std::fmt::Debug> From<T> for Schema {
    fn from(inner: T) -> Self {
        Self {
            inner: inner.into(),
            message: None,
            optional: None,
        }
    }
}

impl Validate for Schema {
    fn validate(&self, value: &reflect::Value) -> Result<(), Error> {
        if let Some(optional) = &self.optional
            && *optional
            && (value.is_null() || value.is_undefined())
        {
            return Ok(());
        }

        if let Err(err) = self.inner.validate(value) {
            if let Some(message) = self.message.clone() {
                return Err(Error::Custom {
                    error: Box::new(err),
                    message,
                });
            } else {
                return Err(err);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnySchema {
    String(StringSchema),
    Number(NumberSchema),
    Integer(IntegerSchema),
    Bool(BoolSchema),
    Null(NullSchema),
    Array(ArraySchema),
    Object(ObjectSchema),
    #[serde(untagged)]
    OneOf {
        oneof: OneOf,
    },
}

impl From<StringSchema> for AnySchema {
    fn from(value: StringSchema) -> Self {
        Self::String(value)
    }
}

impl From<NumberSchema> for AnySchema {
    fn from(value: NumberSchema) -> Self {
        Self::Number(value)
    }
}

impl From<IntegerSchema> for AnySchema {
    fn from(value: IntegerSchema) -> Self {
        Self::Integer(value)
    }
}

impl From<BoolSchema> for AnySchema {
    fn from(value: BoolSchema) -> Self {
        Self::Bool(value)
    }
}

impl From<NullSchema> for AnySchema {
    fn from(value: NullSchema) -> Self {
        Self::Null(value)
    }
}

impl From<ArraySchema> for AnySchema {
    fn from(value: ArraySchema) -> Self {
        Self::Array(value)
    }
}

impl From<ObjectSchema> for AnySchema {
    fn from(value: ObjectSchema) -> Self {
        Self::Object(value)
    }
}

impl From<OneOf> for AnySchema {
    fn from(value: OneOf) -> Self {
        Self::OneOf { oneof: value }
    }
}

impl Validate for AnySchema {
    fn validate(&self, value: &reflect::Value) -> Result<(), Error> {
        match self {
            Self::String(schema) => schema.validate(value),
            Self::Number(schema) => schema.validate(value),
            Self::Integer(schema) => schema.validate(value),
            Self::Bool(schema) => schema.validate(value),
            Self::Null(schema) => schema.validate(value),
            Self::Array(schema) => schema.validate(value),
            Self::Object(schema) => schema.validate(value),
            Self::OneOf { oneof } => oneof.validate(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use reflect::ToValue;

    use crate::{AnySchema, Error, Validate, integer, object, string};

    #[test]
    fn round_trip() -> Result<(), Error> {
        let schema = AnySchema::Object(object().field("name", string().min(1)));
        let json = serde_json::to_string(&schema).unwrap();
        let schema: AnySchema = serde_json::from_str(&json).unwrap();
        let map = reflect::btree_map! {
            "name".to_string() => "nova".to_string()
        };

        let value = map.to_value();
        schema.validate(&value)?;
        Ok(())
    }

    #[test]
    fn round_trip_integer() -> Result<(), Error> {
        let schema = AnySchema::Integer(integer().min(0).max(10));
        let json = serde_json::to_string(&schema).unwrap();
        let schema: AnySchema = serde_json::from_str(&json).unwrap();
        let value = 5_i32.to_value();
        schema.validate(&value)?;
        Ok(())
    }

    fn assert_tag(schema: AnySchema, tag: &str) {
        let json = serde_json::to_string(&schema).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["type"], tag, "wrong tag for {json}");

        let back: AnySchema = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }

    #[test]
    fn every_variant_tagged() {
        use crate::{array, bool, null, number, object, string};

        assert_tag(string().into(), "string");
        assert_tag(number().into(), "number");
        assert_tag(integer().into(), "integer");
        assert_tag(bool().into(), "bool");
        assert_tag(null().into(), "null");
        assert_tag(array().into(), "array");
        assert_tag(object().into(), "object");
    }

    mod optional {
        use reflect::ToValue;

        use crate::{Schema, Validate, string};

        #[test]
        fn skips_null() {
            let schema: Schema = string().into();
            let schema = schema.optional();
            assert!(schema.validate(&reflect::Value::Null).is_ok());
        }

        #[test]
        fn skips_undefined() {
            let schema: Schema = string().into();
            let schema = schema.optional();
            assert!(schema.validate(&reflect::Value::Undefined).is_ok());
        }

        #[test]
        fn still_validates_present_values() {
            let schema: Schema = string().into();
            let schema = schema.optional();
            assert!(schema.validate(&123_i32.to_value()).is_err());
        }

        #[test]
        fn required_rejects_null() {
            let schema: Schema = string().into();
            let schema = schema.required();
            assert!(schema.validate(&reflect::Value::Null).is_err());
        }

        #[test]
        fn default_rejects_null() {
            let schema: Schema = string().into();
            assert!(schema.validate(&reflect::Value::Null).is_err());
        }
    }

    mod message {
        use reflect::ToValue;

        use crate::{Error, Schema, Validate, string};

        #[test]
        fn wraps_error() {
            let schema: Schema = string().into();
            let schema = schema.message("must be a name");
            let err = schema.validate(&123_i32.to_value()).unwrap_err();

            match &err {
                Error::Custom { message, error } => {
                    assert_eq!(message, "must be a name");
                    assert!(!matches!(**error, Error::Custom { .. }));
                }
                other => panic!("expected Error::Custom, got {other:?}"),
            }

            assert_eq!(err.to_string(), "must be a name");
        }

        #[test]
        fn absent_leaves_error_unwrapped() {
            let schema: Schema = string().into();
            let err = schema.validate(&123_i32.to_value()).unwrap_err();
            assert!(!matches!(err, Error::Custom { .. }));
        }

        #[test]
        fn not_applied_on_success() {
            let schema: Schema = string().into();
            let schema = schema.message("must be a name");
            assert!(schema.validate(&"x".to_value()).is_ok());
        }
    }

    mod object_field {
        use reflect::{ToValue, btree_map};

        use crate::{Schema, Validate, object, string};

        #[test]
        fn optional_field_allows_missing_key() {
            let optional: Schema = string().into();
            let schema = object().field("name", optional.optional());
            let empty = btree_map! { "other".to_string() => "ignored".to_string() };
            assert!(schema.validate(&empty.to_value()).is_ok());
        }

        #[test]
        fn required_field_rejects_missing_key() {
            let schema = object().field("name", string());
            let empty = btree_map! { "other".to_string() => "ignored".to_string() };
            assert!(schema.validate(&empty.to_value()).is_err());
        }
    }

    mod serde {
        use reflect::ToValue;

        use crate::{Error, Schema, Validate, string};

        #[test]
        fn optional_survives_round_trip() {
            let schema: Schema = string().into();
            let schema = schema.optional();
            let json = serde_json::to_string(&schema).unwrap();
            let back: Schema = serde_json::from_str(&json).unwrap();
            assert!(back.validate(&reflect::Value::Null).is_ok());
        }

        #[test]
        fn message_survives_round_trip() {
            let schema: Schema = string().into();
            let schema = schema.message("m");
            let json = serde_json::to_string(&schema).unwrap();
            let back: Schema = serde_json::from_str(&json).unwrap();
            let err = back.validate(&123_i32.to_value()).unwrap_err();

            match err {
                Error::Custom { message, .. } => assert_eq!(message, "m"),
                other => panic!("expected Error::Custom, got {other:?}"),
            }
        }
    }
}
