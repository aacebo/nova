use crate::{Error, Validate, error};

pub fn integer() -> IntegerSchema {
    IntegerSchema::default()
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntegerSchema {
    min: Option<i64>,
    max: Option<i64>,
    constant: Option<i64>,
}

impl IntegerSchema {
    pub fn min(mut self, value: i64) -> Self {
        self.min = Some(value);
        self
    }

    pub fn max(mut self, value: i64) -> Self {
        self.max = Some(value);
        self
    }

    pub fn constant(mut self, value: i64) -> Self {
        self.constant = Some(value);
        self
    }
}

impl Validate for IntegerSchema {
    fn validate(&self, value: &reflect::Value) -> Result<(), Error> {
        let mut errors = error::group();
        let number = value
            .as_number()
            .filter(|n| n.is_int())
            .ok_or(("type", format!("expected integer, received {}", value.to_type())))?;
        let int = number.to_i64();

        if let Some(min) = self.min
            && int < min
        {
            errors.push(("min", format!("must not be less than {min}")).into());
        }

        if let Some(max) = self.max
            && int > max
        {
            errors.push(("max", format!("must not be greater than {max}")).into());
        }

        if let Some(constant) = self.constant
            && constant != int
        {
            errors.push(("constant", format!("expected {constant}, received {int}")).into());
        }

        errors.ok()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use reflect::ToValue;

    use crate::{Error, Validate, integer};

    #[test]
    fn fail() {
        let schema = integer();
        let value = 4.5_f64.to_value();
        let res = schema.validate(&value);
        assert!(res.is_err());
    }

    #[test]
    fn succeed() -> Result<(), Error> {
        let schema = integer();
        let value = 42_i32.to_value();
        schema.validate(&value)?;
        Ok(())
    }

    mod min {
        use super::*;

        #[test]
        fn fail() {
            let schema = integer().min(5);
            let value = 4_i32.to_value();
            let res = schema.validate(&value);
            assert!(res.is_err());
        }

        #[test]
        fn succeed() -> Result<(), Error> {
            let schema = integer().min(5);
            let value = 6_i32.to_value();
            schema.validate(&value)?;
            Ok(())
        }
    }

    mod max {
        use super::*;

        #[test]
        fn fail() {
            let schema = integer().max(5);
            let value = 6_i32.to_value();
            let res = schema.validate(&value);
            assert!(res.is_err());
        }

        #[test]
        fn succeed() -> Result<(), Error> {
            let schema = integer().max(5);
            let value = 4_i32.to_value();
            schema.validate(&value)?;
            Ok(())
        }
    }

    mod constant {
        use super::*;

        #[test]
        fn fail() {
            let schema = integer().constant(5);
            let value = 4_i32.to_value();
            let res = schema.validate(&value);
            assert!(res.is_err());
        }

        #[test]
        fn succeed() -> Result<(), Error> {
            let schema = integer().constant(5);
            let value = 5_i32.to_value();
            schema.validate(&value)?;
            Ok(())
        }
    }
}
