use crate::{Error, Schema, Validate};

#[macro_export]
macro_rules! oneof {
    ($($arg:expr),* $(,)?) => {
        $crate::OneOf::new(vec![$($arg.into()),*])
    };
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct OneOf(Vec<Schema>);

impl OneOf {
    pub fn new(schemas: impl IntoIterator<Item = Schema>) -> Self {
        Self(schemas.into_iter().collect())
    }

    pub fn or(mut self, schema: impl Into<Schema>) -> Self {
        self.0.push(schema.into());
        self
    }
}

impl Validate for OneOf {
    fn validate(&self, value: &nova_reflect::Value) -> Result<(), Error> {
        for schema in &self.0 {
            if schema.validate(value).is_ok() {
                return Ok(());
            }
        }

        Err(("oneof", "expected value to match one or more").into())
    }
}

#[cfg(test)]
mod tests {
    use nova_reflect::ToValue;

    use crate::{Error, Validate, bool, string};

    #[test]
    fn fail() {
        let schema = oneof!(string(), bool());
        let value = 1.to_value();
        let res = schema.validate(&value);
        assert!(res.is_err());
    }

    #[test]
    fn succeed() -> Result<(), Error> {
        let schema = oneof!(string(), bool());
        let value = true.to_value();
        schema.validate(&value)?;

        let value = "test".to_value();
        schema.validate(&value)?;
        Ok(())
    }
}
