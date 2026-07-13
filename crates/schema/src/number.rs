use crate::{Error, Validate, error};

pub fn number() -> NumberSchema {
    NumberSchema::default()
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct NumberSchema {
    min: Option<f64>,
    max: Option<f64>,
    constant: Option<f64>,
}

impl NumberSchema {
    pub fn min(mut self, value: f64) -> Self {
        self.min = Some(value);
        self
    }

    pub fn max(mut self, value: f64) -> Self {
        self.max = Some(value);
        self
    }

    pub fn constant(mut self, value: f64) -> Self {
        self.constant = Some(value);
        self
    }
}

impl Validate for NumberSchema {
    fn validate(&self, value: &nova_reflect::Value) -> Result<(), Error> {
        let mut errors = error::group();
        let float = value
            .to_f64()
            .ok_or(("type", format!("expected number, received {}", value.to_type())))?;

        if let Some(min) = self.min
            && float < min
        {
            errors.push(("min", format!("must not be less than {min}")).into());
        }

        if let Some(max) = self.max
            && float > max
        {
            errors.push(("max", format!("must not be greater than {max}")).into());
        }

        if let Some(constant) = self.constant
            && constant != float
        {
            errors.push(("constant", format!("expected {constant}, received {float}")).into());
        }

        errors.ok()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use nova_reflect::ToValue;

    use crate::{Error, Validate, number};

    #[test]
    fn fail() {
        let schema = number();
        let value = "test".to_value();
        let res = schema.validate(&value);
        assert!(res.is_err());
    }

    #[test]
    fn succeed() -> Result<(), Error> {
        let schema = number();
        let value = 42_i32.to_value();
        schema.validate(&value)?;
        Ok(())
    }

    mod min {
        use super::*;

        #[test]
        fn fail() {
            let schema = number().min(5.0);
            let value = 4_i32.to_value();
            let res = schema.validate(&value);
            assert!(res.is_err());
        }

        #[test]
        fn succeed() -> Result<(), Error> {
            let schema = number().min(5.0);
            let value = 6_i32.to_value();
            schema.validate(&value)?;
            Ok(())
        }
    }

    mod max {
        use super::*;

        #[test]
        fn fail() {
            let schema = number().max(5.0);
            let value = 6_i32.to_value();
            let res = schema.validate(&value);
            assert!(res.is_err());
        }

        #[test]
        fn succeed() -> Result<(), Error> {
            let schema = number().max(5.0);
            let value = 4_i32.to_value();
            schema.validate(&value)?;
            Ok(())
        }
    }

    mod constant {
        use super::*;

        #[test]
        fn fail() {
            let schema = number().constant(5.0);
            let value = 4_i32.to_value();
            let res = schema.validate(&value);
            assert!(res.is_err());
        }

        #[test]
        fn succeed() -> Result<(), Error> {
            let schema = number().constant(5.0);
            let value = 5_i32.to_value();
            schema.validate(&value)?;
            Ok(())
        }
    }

    mod serde {
        use super::*;

        #[test]
        fn round_trip() -> Result<(), Error> {
            let schema = number().min(0.0).max(10.0);
            let json = serde_json::to_string(&schema).unwrap();
            let schema: crate::NumberSchema = serde_json::from_str(&json).unwrap();

            schema.validate(&5_i32.to_value())?;
            assert!(schema.validate(&11_i32.to_value()).is_err());
            Ok(())
        }
    }
}
