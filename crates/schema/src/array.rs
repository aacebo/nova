use crate::{Error, Schema, Validate, error};

pub fn array() -> ArraySchema {
    ArraySchema::default()
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArraySchema {
    min: Option<usize>,
    max: Option<usize>,
    items: Option<Box<Schema>>,
}

impl ArraySchema {
    pub fn min(mut self, value: usize) -> Self {
        self.min = Some(value);
        self
    }

    pub fn max(mut self, value: usize) -> Self {
        self.max = Some(value);
        self
    }

    pub fn items(mut self, schema: impl Into<Schema>) -> Self {
        self.items = Some(Box::new(schema.into()));
        self
    }
}

impl Validate for ArraySchema {
    fn validate(&self, value: &reflect::Value) -> Result<(), Error> {
        let mut errors = error::group();
        let sequence = value
            .as_dynamic()
            .and_then(|d| d.as_sequence())
            .ok_or(("type", format!("expected array, received {}", value.to_type())))?;
        let len = sequence.len();

        if let Some(min) = self.min
            && len < min
        {
            errors.push(("min", format!("length must not be less than {min}")).into());
        }

        if let Some(max) = self.max
            && len > max
        {
            errors.push(("max", format!("length must not be greater than {max}")).into());
        }

        if let Some(items) = &self.items {
            let mut list = error::list();

            for i in 0..len {
                if let Err(err) = items.validate(&sequence.index(i)) {
                    list = list.index(i, err);
                }
            }

            if !list.is_empty() {
                errors.push(list.into());
            }
        }

        errors.ok()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use reflect::ToValue;

    use crate::{Error, Schema, Validate, array, number};

    #[test]
    fn fail() {
        let schema = array();
        let value = "test".to_value();
        let res = schema.validate(&value);
        assert!(res.is_err());
    }

    #[test]
    fn succeed() -> Result<(), Error> {
        let schema = array();
        let v = vec![1_i32, 2, 3];
        let value = v.to_value();
        schema.validate(&value)?;
        Ok(())
    }

    mod min {
        use super::*;

        #[test]
        fn fail() {
            let schema = array().min(3);
            let v = vec![1_i32, 2];
            let value = v.to_value();
            let res = schema.validate(&value);
            assert!(res.is_err());
        }

        #[test]
        fn succeed() -> Result<(), Error> {
            let schema = array().min(3);
            let v = vec![1_i32, 2, 3];
            let value = v.to_value();
            schema.validate(&value)?;
            Ok(())
        }
    }

    mod max {
        use super::*;

        #[test]
        fn fail() {
            let schema = array().max(2);
            let v = vec![1_i32, 2, 3];
            let value = v.to_value();
            let res = schema.validate(&value);
            assert!(res.is_err());
        }

        #[test]
        fn succeed() -> Result<(), Error> {
            let schema = array().max(3);
            let v = vec![1_i32, 2, 3];
            let value = v.to_value();
            schema.validate(&value)?;
            Ok(())
        }
    }

    mod items {
        use super::*;

        #[test]
        fn fail() {
            let schema = array().items(number().min(10.0));
            let v = vec![1_i32, 2, 3];
            let value = v.to_value();
            let res = schema.validate(&value);
            assert!(res.is_err());
        }

        #[test]
        fn succeed() -> Result<(), Error> {
            let schema = array().items(number());
            let v = vec![1_i32, 2, 3];
            let value = v.to_value();
            schema.validate(&value)?;
            Ok(())
        }
    }
}
