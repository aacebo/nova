use crate::{Error, Validate, error};

pub fn number() -> NumberSchema {
    NumberSchema::default()
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct NumberSchema {
    min: Option<f64>,
    max: Option<f64>,
    integer: Option<bool>,
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

    pub fn integer(mut self, value: bool) -> Self {
        self.integer = Some(value);
        self
    }
}

impl Validate for NumberSchema {
    fn validate(&self, value: &reflect::Value) -> Result<(), Error> {
        let mut errors = error::group();
        let number = value
            .as_number()
            .ok_or(("type", format!("expected number, received {}", value.to_type())))?;
        let float = to_f64(number);

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

        if self.integer == Some(true) && !number.is_int() {
            errors.push(("integer", "expected an integer".to_string()).into());
        }

        errors.ok()?;
        Ok(())
    }
}

fn to_f64(number: &reflect::Number) -> f64 {
    match number {
        reflect::Number::Int(v) => match v {
            reflect::Int::I8(n) => *n as f64,
            reflect::Int::I16(n) => *n as f64,
            reflect::Int::I32(n) => *n as f64,
            reflect::Int::I64(n) => *n as f64,
            reflect::Int::U8(n) => *n as f64,
            reflect::Int::U16(n) => *n as f64,
            reflect::Int::U32(n) => *n as f64,
            reflect::Int::U64(n) => *n as f64,
        },
        reflect::Number::Float(v) => match v {
            reflect::Float::F32(n) => *n as f64,
            reflect::Float::F64(n) => *n,
        },
    }
}

#[cfg(test)]
mod tests {
    use reflect::ToValue;

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

    mod integer {
        use super::*;

        #[test]
        fn fail() {
            let schema = number().integer(true);
            let value = 4.5_f64.to_value();
            let res = schema.validate(&value);
            assert!(res.is_err());
        }

        #[test]
        fn succeed() -> Result<(), Error> {
            let schema = number().integer(true);
            let value = 4_i32.to_value();
            schema.validate(&value)?;
            Ok(())
        }
    }
}
