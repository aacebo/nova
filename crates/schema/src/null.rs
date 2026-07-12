use crate::{Error, Validate};

pub fn null() -> NullSchema {
    NullSchema::default()
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct NullSchema;

impl Validate for NullSchema {
    fn validate(&self, value: &reflect::Value) -> Result<(), Error> {
        if !value.is_null() && !value.is_undefined() {
            return Err(("type", format!("expected null, received {}", value.to_type())).into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use reflect::ToValue;

    use crate::{Error, Validate, null};

    #[test]
    fn fail() {
        let schema = null();
        let value = true.to_value();
        let res = schema.validate(&value);
        assert!(res.is_err());
    }

    #[test]
    fn succeed() -> Result<(), Error> {
        let schema = null();
        let value = reflect::Value::Null;
        schema.validate(&value)?;
        Ok(())
    }
}
