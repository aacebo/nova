use std::collections::BTreeMap;

use crate::Value;

#[derive(Default, Debug, Clone, serde::Serialize)]
#[serde(transparent)]
pub struct Args(BTreeMap<String, Value>);

impl Args {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut Value)> {
        self.0.iter_mut()
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.0.get_mut(key)
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<Value>) -> &mut Self {
        self.0.insert(key.into(), value.into());
        self
    }
}

impl<K: Into<String>, V: Into<Value>, T: IntoIterator<Item = (K, V)>> From<T> for Args {
    fn from(value: T) -> Self {
        let mut items = BTreeMap::new();

        for (k, v) in value.into_iter() {
            items.insert(k.into(), v.into());
        }

        Self(items)
    }
}

impl<K: Into<String>, V: Into<Value>> FromIterator<(K, V)> for Args {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut items = BTreeMap::new();

        for (k, v) in iter {
            items.insert(k.into(), v.into());
        }

        Self(items)
    }
}

impl std::ops::Index<&str> for Args {
    type Output = Value;

    fn index(&self, index: &str) -> &Self::Output {
        self.0.index(index)
    }
}
