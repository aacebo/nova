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
    fn name(&self) -> &str;
    fn validate(&self, value: &reflect::Value) -> Result<(), Error>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Schema {
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

impl From<StringSchema> for Schema {
    fn from(value: StringSchema) -> Self {
        Self::String(value)
    }
}

impl From<NumberSchema> for Schema {
    fn from(value: NumberSchema) -> Self {
        Self::Number(value)
    }
}

impl From<IntegerSchema> for Schema {
    fn from(value: IntegerSchema) -> Self {
        Self::Integer(value)
    }
}

impl From<BoolSchema> for Schema {
    fn from(value: BoolSchema) -> Self {
        Self::Bool(value)
    }
}

impl From<NullSchema> for Schema {
    fn from(value: NullSchema) -> Self {
        Self::Null(value)
    }
}

impl From<ArraySchema> for Schema {
    fn from(value: ArraySchema) -> Self {
        Self::Array(value)
    }
}

impl From<ObjectSchema> for Schema {
    fn from(value: ObjectSchema) -> Self {
        Self::Object(value)
    }
}

impl From<OneOf> for Schema {
    fn from(value: OneOf) -> Self {
        Self::OneOf { oneof: value }
    }
}

impl Validate for Schema {
    fn name(&self) -> &str {
        match self {
            Self::String(v) => v.name(),
            Self::Number(v) => v.name(),
            Self::Integer(v) => v.name(),
            Self::Bool(v) => v.name(),
            Self::Null(v) => v.name(),
            Self::Array(v) => v.name(),
            Self::Object(v) => v.name(),
            Self::OneOf { oneof } => oneof.name(),
        }
    }

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

    use crate::{Error, Schema, Validate, integer, object, string};

    #[test]
    fn round_trip() -> Result<(), Error> {
        let schema = Schema::Object(object().field("name", string().min(1)));
        let json = serde_json::to_string(&schema).unwrap();
        let schema: Schema = serde_json::from_str(&json).unwrap();
        let map = reflect::btree_map! {
            "name".to_string() => "nova".to_string()
        };

        let value = map.to_value();
        schema.validate(&value)?;
        Ok(())
    }

    #[test]
    fn round_trip_integer() -> Result<(), Error> {
        let schema = Schema::Integer(integer().min(0).max(10));
        let json = serde_json::to_string(&schema).unwrap();
        let schema: Schema = serde_json::from_str(&json).unwrap();
        let value = 5_i32.to_value();
        schema.validate(&value)?;
        Ok(())
    }

    fn assert_tag(schema: Schema, tag: &str) {
        let json = serde_json::to_string(&schema).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["type"], tag, "wrong tag for {json}");

        let back: Schema = serde_json::from_str(&json).unwrap();
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
}
