use std::collections::BTreeMap;

use nova_reflect::{Value, ValueRef};

pub trait FromArgs {
    type Error;

    fn from_args(args: &Args) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

#[derive(Default, Debug, Clone, serde::Serialize)]
pub struct Args {
    args: Vec<Value>,
    kargs: KArgs,
}

impl Args {
    pub fn new(args: impl IntoIterator<Item = Value>, kargs: KArgs) -> Self {
        Self {
            args: args.into_iter().collect(),
            kargs,
        }
    }

    pub fn args(&self) -> &[Value] {
        &self.args
    }

    pub fn kargs(&self) -> &KArgs {
        &self.kargs
    }

    pub fn at(&self, index: usize) -> Value {
        self.args.get(index).cloned().unwrap_or(Value::Undefined)
    }

    pub fn key(&self, key: impl AsRef<str>) -> Value {
        self.kargs.get(key.as_ref()).cloned().unwrap_or(Value::Undefined)
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.args.get(index)
    }

    pub fn str(&self, index: usize) -> Option<String> {
        self.args.get(index).and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    pub fn key_str(&self, key: impl AsRef<str>) -> Option<String> {
        self.kargs.get(key.as_ref()).and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    pub fn u64(&self, index: usize) -> Option<u64> {
        self.at(index).to_u64()
    }

    pub fn key_u64(&self, key: impl AsRef<str>) -> Option<u64> {
        self.key(key).to_u64()
    }

    pub fn f64(&self, index: usize) -> Option<f64> {
        self.at(index).to_f64()
    }

    pub fn key_f64(&self, key: impl AsRef<str>) -> Option<f64> {
        self.key(key).to_f64()
    }

    pub fn bool(&self, index: usize) -> bool {
        self.at(index).as_ref().is_truthy()
    }

    pub fn key_bool(&self, key: impl AsRef<str>) -> bool {
        self.key(key).as_ref().is_truthy()
    }

    pub fn get_key(&self, key: impl AsRef<str>) -> Option<&Value> {
        self.kargs.get(key.as_ref())
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty() && self.kargs.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Value, ValueRef<'_>)> {
        use nova_reflect::{Int, Number, Str};

        let positional = self
            .args
            .iter()
            .enumerate()
            .map(|(i, v)| (Value::Number(Number::Int(Int::U64(i as u64))), v.as_ref()));

        let keyword = self
            .kargs
            .iter()
            .map(|(k, v)| (Value::Str(Str::from(k.as_str())), v.as_ref()));

        positional.chain(keyword)
    }
}

impl From<&Args> for Args {
    fn from(value: &Args) -> Self {
        value.clone()
    }
}

impl nova_reflect::TypeOf for Args {
    fn type_of() -> nova_reflect::Type {
        nova_reflect::Type::Any
    }
}

impl nova_reflect::ToType for Args {
    fn to_type(&self) -> nova_reflect::Type {
        nova_reflect::Type::Any
    }
}

impl nova_reflect::Sequence for Args {
    fn len(&self) -> usize {
        self.args.len()
    }

    fn index_by_ref(&self, i: usize) -> ValueRef<'_> {
        match self.args.get(i) {
            Some(value) => value.as_ref(),
            None => ValueRef::Undefined,
        }
    }
}

impl std::ops::Deref for Args {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.args
    }
}

impl std::ops::Index<usize> for Args {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        self.args.index(index)
    }
}

impl std::ops::Index<&str> for Args {
    type Output = Value;

    fn index(&self, index: &str) -> &Self::Output {
        self.kargs.index(index)
    }
}

#[derive(Default, Debug, Clone, serde::Serialize)]
#[serde(transparent)]
pub struct KArgs(BTreeMap<String, Value>);

impl KArgs {
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

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.0.get_mut(key)
    }

    pub fn get_required<T>(&self, key: &str) -> Result<T, crate::Error>
    where
        T: TryFrom<Value>,
        T::Error: std::fmt::Display,
    {
        let value = self
            .0
            .get(key)
            .cloned()
            .ok_or_else(|| crate::Error::message(format!("missing required argument `{key}`")))?;

        T::try_from(value).map_err(|err| crate::Error::message(err.to_string()))
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

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<Value>) -> &mut Self {
        self.0.insert(key.into(), value.into());
        self
    }

    pub fn into_inner(self) -> BTreeMap<String, Value> {
        self.0
    }
}

impl<K: Into<String>, V: Into<Value>> FromIterator<(K, V)> for KArgs {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self(iter.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
}

impl std::ops::Index<&str> for KArgs {
    type Output = Value;

    fn index(&self, index: &str) -> &Self::Output {
        self.0.index(index)
    }
}
