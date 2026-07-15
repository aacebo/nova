#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Str(pub std::sync::Arc<str>);

impl Str {
    pub fn new(value: impl Into<std::sync::Arc<str>>) -> Self {
        Self(value.into())
    }

    pub fn to_type(&self) -> crate::Type {
        crate::Type::Str(crate::StrType)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl AsRef<str> for Str {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for Str {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Str {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl PartialEq<std::string::String> for Str {
    fn eq(&self, other: &std::string::String) -> bool {
        self.0.as_ref() == other.as_str()
    }
}

impl PartialEq<str> for Str {
    fn eq(&self, other: &str) -> bool {
        self.0.as_ref() == other
    }
}

impl From<std::string::String> for Str {
    fn from(value: std::string::String) -> Self {
        Self(std::sync::Arc::from(value))
    }
}

impl From<&str> for Str {
    fn from(value: &str) -> Self {
        Self(std::sync::Arc::from(value))
    }
}

impl From<std::string::String> for crate::Value {
    fn from(value: std::string::String) -> Self {
        crate::Value::Str(Str::from(value))
    }
}

impl From<&str> for crate::Value {
    fn from(value: &str) -> Self {
        crate::Value::Str(Str::from(value))
    }
}

impl crate::ToValue for std::string::String {
    fn to_value_ref(&self) -> crate::ValueRef<'_> {
        crate::ValueRef::Str(self.as_str())
    }
}

impl crate::ToValue for &'static str {
    fn to_value_ref(&self) -> crate::ValueRef<'static> {
        crate::ValueRef::Str(self)
    }
}

impl crate::ToValue for str {
    fn to_value_ref(&self) -> crate::ValueRef<'_> {
        crate::ValueRef::Str(self)
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    pub fn str() {
        let value = value_of!("test");

        assert!(value.is_str());
        assert_eq!(value.len(), 4);
        assert_eq!(value.to_string(), "test");
    }

    #[test]
    pub fn string() {
        let s = "test".to_string();
        let value = value_of!(s);

        assert!(value.is_str());
        assert_eq!(value.len(), 4);
        assert_eq!(value.to_string(), "test");
    }
}
