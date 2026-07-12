mod array;
mod bool;
pub mod error;
mod null;
mod number;
mod object;
mod string;

pub use array::*;
pub use bool::*;
pub use error::Error;
pub use null::*;
pub use number::*;
pub use object::*;
pub use string::*;

pub trait Validate {
    fn validate(&self, value: &reflect::Value) -> Result<(), Error>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Schema {
    String(StringSchema),
    Number(NumberSchema),
    Bool(BoolSchema),
    Null(NullSchema),
    Array(ArraySchema),
    Object(ObjectSchema),
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

impl Validate for Schema {
    fn validate(&self, value: &reflect::Value) -> Result<(), Error> {
        match self {
            Self::String(schema) => schema.validate(value),
            Self::Number(schema) => schema.validate(value),
            Self::Bool(schema) => schema.validate(value),
            Self::Null(schema) => schema.validate(value),
            Self::Array(schema) => schema.validate(value),
            Self::Object(schema) => schema.validate(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use reflect::ToValue;

    use crate::{Error, Schema, Validate, object, string};

    #[test]
    fn round_trip() -> Result<(), Error> {
        let schema = Schema::Object(object().field("name", Schema::String(string().min(1))));
        let json = serde_json::to_string(&schema).unwrap();
        let schema: Schema = serde_json::from_str(&json).unwrap();
        let map = reflect::btree_map! {
            "name".to_string() => "nova".to_string()
        };

        let value = map.to_value();
        schema.validate(&value)?;
        Ok(())
    }
}
