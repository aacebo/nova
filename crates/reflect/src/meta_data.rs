use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(transparent))]
pub struct MetaData(BTreeMap<String, Arc<crate::Value>>);

impl MetaData {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> std::collections::btree_map::Iter<'_, String, Arc<crate::Value>> {
        self.0.iter()
    }

    pub fn has(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub fn get(&self, key: &str) -> Option<&crate::Value> {
        self.0.get(key).map(Arc::as_ref)
    }

    pub fn merge(mut self, other: &Self) -> Self {
        for (key, value) in &other.0 {
            self.0.insert(key.clone(), value.clone());
        }

        self
    }
}

impl<const N: usize, V: Into<crate::Value>> From<[(&str, V); N]> for MetaData {
    fn from(items: [(&str, V); N]) -> Self {
        let mut data = BTreeMap::new();

        for (key, value) in items {
            data.insert(key.to_string(), Arc::new(value.into()));
        }

        Self(data)
    }
}

impl std::ops::Index<&str> for MetaData {
    type Output = crate::Value;

    fn index(&self, index: &str) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl std::fmt::Display for MetaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;

        for (key, value) in &self.0 {
            write!(f, "\n\t{}: {}", key, value)?;
        }

        if !self.0.is_empty() {
            writeln!(f)?;
        }

        write!(f, "}}")
    }
}

impl crate::TypeOf for MetaData {
    fn type_of() -> crate::Type {
        crate::struct_type()
            .path(crate::Path::from("nova_reflect"))
            .name("MetaData")
            .visibility(crate::Visibility::Public(crate::Public::Full))
            .build()
            .to_type()
    }
}

impl crate::ToType for MetaData {
    fn to_type(&self) -> crate::Type {
        <Self as crate::TypeOf>::type_of()
    }
}

impl crate::ToValue for MetaData {
    fn to_value_ref(&self) -> crate::ValueRef<'_> {
        crate::ValueRef::Dynamic(crate::DynamicRef::from_object(self))
    }
}

impl crate::Object for MetaData {
    fn field(&self, name: &str) -> crate::ValueRef<'_> {
        self.get(name).map(crate::Value::as_ref).unwrap_or(crate::ValueRef::Undefined)
    }
}
