use crate::{Error, Validate, error};

pub fn string() -> StringSchema {
    StringSchema::default()
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct StringSchema {
    min: Option<usize>,
    max: Option<usize>,
    constant: Option<String>,
    #[serde(with = "pattern", default)]
    pattern: Option<regex::Regex>,
}

impl StringSchema {
    pub fn min(mut self, value: usize) -> Self {
        self.min = Some(value);
        self
    }

    pub fn max(mut self, value: usize) -> Self {
        self.max = Some(value);
        self
    }

    pub fn constant(mut self, value: impl Into<String>) -> Self {
        self.constant = Some(value.into());
        self
    }

    pub fn pattern(mut self, value: impl std::fmt::Display) -> Result<Self, regex::Error> {
        self.pattern = Some(value.to_string().parse().unwrap());
        Ok(self)
    }
}

impl Validate for StringSchema {
    fn validate(&self, value: &reflect::Value) -> Result<(), Error> {
        let mut errors = error::group();
        let value = value
            .as_str()
            .ok_or(("type", format!("expected string, received {}", value.to_type())))?;

        if let Some(min) = self.min
            && min > value.len()
        {
            errors.push(("min", format!("length must not be less than {min}")).into());
        }

        if let Some(max) = self.max
            && max < value.len()
        {
            errors.push(("max", format!("length must not be greater than {max}")).into());
        }

        if let Some(constant) = &self.constant
            && value != constant
        {
            errors.push(("constant", format!("expected \"{constant}\", received \"{value}\"")).into());
        }

        if let Some(pattern) = &self.pattern
            && !pattern.is_match(value)
        {
            errors.push(("pattern", format!("expected match for pattern \"{pattern}\"")).into());
        }

        errors.ok()?;
        Ok(())
    }
}

mod pattern {
    pub fn serialize<S>(value: &Option<regex::Regex>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(expr) = value {
            serializer.serialize_some(expr.as_str())
        } else {
            serializer.serialize_none()
        }
    }

    // The deserialize function must match this exact signature
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<regex::Regex>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::Deserialize;

        let value = Option::<String>::deserialize(deserializer)?;

        match value {
            Some(s) => {
                let re = regex::Regex::new(&s).map_err(serde::de::Error::custom)?;
                Ok(Some(re))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use reflect::ToValue;

    use crate::{Error, Validate, string};

    #[test]
    fn fail() {
        let schema = string();
        let value = true.to_value();
        let res = schema.validate(&value);
        assert!(res.is_err());
    }

    #[test]
    fn succeed() -> Result<(), Error> {
        let schema = string();
        let value = "test".to_value();
        schema.validate(&value)?;
        Ok(())
    }

    mod min {
        use super::*;

        #[test]
        fn fail() {
            let schema = string().min(5);
            let value = "test".to_value();
            let res = schema.validate(&value);
            assert!(res.is_err());
        }

        #[test]
        fn succeed() -> Result<(), Error> {
            let schema = string().min(5);
            let value = "tester".to_value();
            schema.validate(&value)?;
            Ok(())
        }
    }

    mod max {
        use super::*;

        #[test]
        fn fail() {
            let schema = string().max(5);
            let value = "tester".to_value();
            let res = schema.validate(&value);
            assert!(res.is_err());
        }

        #[test]
        fn succeed() -> Result<(), Error> {
            let schema = string().max(5);
            let value = "test".to_value();
            schema.validate(&value)?;
            Ok(())
        }
    }

    mod constant {
        use super::*;

        #[test]
        fn fail() {
            let schema = string().constant("nova");
            let value = "test".to_value();
            let res = schema.validate(&value);
            assert!(res.is_err());
        }

        #[test]
        fn succeed() -> Result<(), Error> {
            let schema = string().constant("nova");
            let value = "nova".to_value();
            schema.validate(&value)?;
            Ok(())
        }
    }

    mod pattern {
        use super::*;

        #[test]
        fn fail() {
            let schema = string().pattern("^[0-9]+$").unwrap();
            let value = "123a".to_value();
            let res = schema.validate(&value);
            assert!(res.is_err());
        }

        #[test]
        fn succeed() -> Result<(), Error> {
            let schema = string().pattern("^[0-9]+$").unwrap();
            let value = "123".to_value();
            schema.validate(&value)?;
            Ok(())
        }
    }
}
