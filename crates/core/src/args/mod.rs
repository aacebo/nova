mod kargs;

pub use kargs::*;

use crate::{Error, Value};

pub trait FromArgs {
    type Error;

    fn from_args(args: &Args<'_>) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

pub struct Args<'a> {
    pub args: &'a [Value],
    pub kargs: KArgs,
}

impl<'a> Args<'a> {
    pub fn new(args: &'a [Value], kargs: KArgs) -> Self {
        Self { args, kargs }
    }

    pub fn args(&self) -> &[Value] {
        self.args
    }

    pub fn kargs(&self) -> &KArgs {
        &self.kargs
    }

    pub fn at(&self, index: usize) -> Value {
        self.args.get(index).cloned().unwrap_or_default()
    }

    pub fn key(&self, key: impl AsRef<str>) -> Value {
        self.kargs.get(key.as_ref()).cloned().unwrap_or_default()
    }

    pub fn to_tuple(&'a self) -> (&'a [Value], &'a KArgs) {
        (self.args, &self.kargs)
    }

    pub fn into_tuple(self) -> (&'a [Value], KArgs) {
        (self.args, self.kargs)
    }
}

impl<'a> TryFrom<&'a [Value]> for Args<'a> {
    type Error = Error;

    fn try_from(values: &'a [Value]) -> Result<Self, Self::Error> {
        let (args, kwargs): (&[Value], minijinja::value::Kwargs) = minijinja::value::from_args(values)?;

        Ok(Self {
            args,
            kargs: KArgs::from_kwargs(kwargs)?,
        })
    }
}

pub struct ArgsIter<'a> {
    args: std::iter::Enumerate<std::slice::Iter<'a, Value>>,
    kargs: Box<dyn Iterator<Item = (&'a String, &'a Value)> + 'a>,
}

impl Iterator for ArgsIter<'_> {
    type Item = (Value, Value);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((index, value)) = self.args.next() {
            return Some((Value::from(index), value.clone()));
        }

        self.kargs
            .next()
            .map(|(key, value)| (Value::from(key.clone()), value.clone()))
    }
}

impl<'a> IntoIterator for &'a Args<'a> {
    type Item = (Value, Value);
    type IntoIter = ArgsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ArgsIter {
            args: self.args.iter().enumerate(),
            kargs: Box::new(self.kargs.iter()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iterates_positional_then_keyword() {
        let positional = [Value::from("a"), Value::from("b")];
        let kargs = KArgs::from_iter([("name", Value::from("x"))]);
        let args = Args::new(&positional, kargs);

        let pairs: Vec<(Value, Value)> = args.into_iter().collect();

        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0], (Value::from(0usize), Value::from("a")));
        assert_eq!(pairs[1], (Value::from(1usize), Value::from("b")));
        assert_eq!(pairs[2], (Value::from("name"), Value::from("x")));
    }
}
