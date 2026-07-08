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

    pub fn push(&mut self, value: impl Into<Value>) -> &mut Self {
        let idx = self.0.keys().filter(|k| k.parse::<usize>().is_ok()).count();
        self.0.insert(idx.to_string(), value.into());
        self
    }

    pub fn at(&self, n: usize) -> Option<&Value> {
        self.0.get(&n.to_string())
    }

    pub fn get_required<T>(&self, key: &str) -> Result<T, Box<dyn std::error::Error>>
    where
        T: TryFrom<Value>,
        T::Error: std::error::Error + 'static,
    {
        let value = self
            .0
            .get(key)
            .cloned()
            .ok_or_else(|| crate::Error::message(format!("missing required argument `{key}`")))?;

        Ok(T::try_from(value)?)
    }

    pub fn get_or_default<T>(&self, key: &str) -> T
    where
        T: TryFrom<Value> + Default,
    {
        match self.0.get(key).cloned() {
            Some(value) => T::try_from(value).unwrap_or_default(),
            None => T::default(),
        }
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

impl Args {
    /// Consume minijinja keyword arguments into `Args`, asserting all were used.
    pub fn from_kwargs(kwargs: minijinja::value::Kwargs) -> Result<Self, minijinja::Error> {
        let mut args = Self::new();

        for key in kwargs.args() {
            args.set(key, kwargs.get::<Value>(key)?);
        }

        kwargs.assert_all_used()?;
        Ok(args)
    }
}

impl From<Args> for Value {
    fn from(args: Args) -> Self {
        Value::from_iter(args.0)
    }
}
