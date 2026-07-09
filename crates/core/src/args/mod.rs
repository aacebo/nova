mod kargs;

pub use kargs::*;

use crate::{Error, Value};

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
