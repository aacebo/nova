use std::collections::BTreeMap;

use crate::Pointer;

pub trait FromArgs {
    type Error;

    fn from_args(args: &Args) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

#[derive(Default, Debug, Clone, serde::Serialize)]
#[serde(transparent)]
pub struct KArgs(BTreeMap<String, Pointer>);

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

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Pointer)> {
        self.0.iter()
    }

    pub fn get(&self, key: &str) -> Option<&Pointer> {
        self.0.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Pointer> {
        self.0.get_mut(key)
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<Pointer>) -> &mut Self {
        self.0.insert(key.into(), value.into());
        self
    }

    pub fn into_inner(self) -> BTreeMap<String, Pointer> {
        self.0
    }
}

impl<K: Into<String>, V: Into<Pointer>> FromIterator<(K, V)> for KArgs {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self(iter.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
}

impl std::ops::Index<&str> for KArgs {
    type Output = Pointer;

    fn index(&self, index: &str) -> &Self::Output {
        self.0.index(index)
    }
}

#[derive(Default, Debug, Clone)]
pub struct Args {
    args: Vec<Pointer>,
    kargs: KArgs,
    caller: Option<Pointer>,
}

impl Args {
    pub fn new(args: impl IntoIterator<Item = Pointer>, kargs: KArgs) -> Self {
        Self {
            args: args.into_iter().collect(),
            kargs,
            caller: None,
        }
    }

    pub fn with_caller(mut self, caller: Pointer) -> Self {
        self.caller = Some(caller);
        self
    }

    pub fn args(&self) -> &[Pointer] {
        &self.args
    }

    pub fn kargs(&self) -> &KArgs {
        &self.kargs
    }

    pub fn caller(&self) -> Option<&Pointer> {
        self.caller.as_ref()
    }

    pub fn at(&self, index: usize) -> Pointer {
        self.args
            .get(index)
            .cloned()
            .unwrap_or_else(|| Pointer::new(nova_reflect::Value::Undefined))
    }

    pub fn key(&self, key: impl AsRef<str>) -> Pointer {
        self.kargs
            .get(key.as_ref())
            .cloned()
            .unwrap_or_else(|| Pointer::new(nova_reflect::Value::Undefined))
    }

    pub fn get(&self, index: usize) -> Option<&Pointer> {
        self.args.get(index)
    }

    pub fn get_key(&self, key: impl AsRef<str>) -> Option<&Pointer> {
        self.kargs.get(key.as_ref())
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty() && self.kargs.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (nova_reflect::Value<'_>, nova_reflect::Value<'_>)> {
        use nova_reflect::{Int, Number, Str, Value};

        let positional = self
            .args
            .iter()
            .enumerate()
            .map(|(i, v)| (Value::Number(Number::Int(Int::U64(i as u64))), v.value()));

        let keyword = self
            .kargs
            .iter()
            .map(|(k, v)| (Value::Str(Str(std::borrow::Cow::Borrowed(k.as_str()))), v.value()));

        positional.chain(keyword)
    }
}
