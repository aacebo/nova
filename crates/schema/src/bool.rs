use crate::{Error, Validate, error};

pub fn bool() -> BoolSchema {
    BoolSchema::default()
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct BoolSchema {
    value: Option<bool>,
}

impl BoolSchema {
    pub fn constant(mut self, value: bool) -> Self {
        self.value = Some(value);
        self
    }
}

impl Validate for BoolSchema {
    fn validate(&self, value: &reflect::Value) -> Result<(), Error> {
        let mut errors = error::group();
        let value = value
            .as_bool()
            .ok_or(("type", format!("expected bool, received {}", value.to_type())))?;

        if let Some(v) = &self.value
            && v != value
        {
            errors.push(("constant", format!("expected {v}, received {value}")).into());
        }

        errors.ok()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use reflect::ToValue;

    use crate::{Error, Validate, bool};

    #[test]
    fn fail() {
        let schema = bool();
        let value = 1.to_value();
        let res = schema.validate(&value);
        assert!(res.is_err());
    }

    #[test]
    fn succeed() -> Result<(), Error> {
        let schema = bool();
        let value = true.to_value();
        schema.validate(&value)?;
        Ok(())
    }

    mod constant {
        use super::*;

        #[test]
        fn fail() {
            let schema = bool().constant(true);
            let value = false.to_value();
            let res = schema.validate(&value);
            assert!(res.is_err());
        }

        #[test]
        fn succeed() -> Result<(), Error> {
            let schema = bool().constant(true);
            let value = true.to_value();
            schema.validate(&value)?;
            Ok(())
        }
    }
}
