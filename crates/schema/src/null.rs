use crate::{Error, Validate};

pub fn null() -> NullSchema {
    NullSchema
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct NullSchema;

impl Validate for NullSchema {
    fn validate(&self, value: &nova_reflect::Value) -> Result<(), Error> {
        if !value.is_null() && !value.is_undefined() {
            return Err(("type", format!("expected null, received {}", value.to_type())).into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use nova_reflect::ToValue;

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
        let value = nova_reflect::Value::Null;
        schema.validate(&value)?;
        Ok(())
    }

    mod serde {
        use super::*;

        #[test]
        fn round_trip() -> Result<(), Error> {
            let json = serde_json::to_string(&null()).unwrap();
            let schema: crate::NullSchema = serde_json::from_str(&json).unwrap();

            schema.validate(&nova_reflect::Value::Null)?;
            assert!(schema.validate(&true.to_value()).is_err());
            Ok(())
        }
    }
}
