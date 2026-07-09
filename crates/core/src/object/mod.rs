mod function;
mod routine;

pub use function::*;
pub use routine::*;

use crate::{Action, Call, Predicate, Value};

#[derive(Debug, Clone)]
pub enum Object {
    Func(Function),
    Routine(Routine),
    Value(Value),
}

impl Object {
    pub fn action(name: impl Into<String>, action: impl Action + 'static) -> Self {
        Self::Func(Function::action(name, action))
    }

    pub fn predicate(name: impl Into<String>, predicate: impl Predicate + 'static) -> Self {
        Self::Func(Function::predicate(name, predicate))
    }

    pub fn func(name: impl Into<String>, func: impl Call + 'static) -> Self {
        Self::Func(Function::func(name, func))
    }

    pub fn routine(routine: Routine) -> Self {
        Self::Routine(routine)
    }

    pub fn value(value: Value) -> Self {
        Self::Value(value)
    }

    pub fn is_func(&self) -> bool {
        matches!(self, Self::Func(_))
    }

    pub fn is_routine(&self) -> bool {
        matches!(self, Self::Routine(_))
    }

    pub fn is_value(&self) -> bool {
        matches!(self, Self::Value(_))
    }

    pub fn as_func(&self) -> Option<&Function> {
        match self {
            Self::Func(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_routine(&self) -> Option<&Routine> {
        match self {
            Self::Routine(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_value(&self) -> Option<&Value> {
        match self {
            Self::Value(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_value_mut(&mut self) -> Option<&mut Value> {
        match self {
            Self::Value(v) => Some(v),
            _ => None,
        }
    }
}

impl From<Function> for Object {
    fn from(value: Function) -> Self {
        Self::Func(value)
    }
}

impl From<Routine> for Object {
    fn from(value: Routine) -> Self {
        Self::Routine(value)
    }
}

impl<T: Into<Value>> From<T> for Object {
    fn from(value: T) -> Self {
        Self::Value(value.into())
    }
}

impl Eq for Object {}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Func(a), Self::Func(b)) => a.name() == b.name(),
            (Self::Routine(a), Self::Routine(b)) => a.name() == b.name(),
            (Self::Value(a), Self::Value(b)) => *a == *b,
            _ => false,
        }
    }
}

impl PartialEq<Value> for Object {
    fn eq(&self, other: &Value) -> bool {
        if let Some(value) = self.as_value() {
            *value == *other
        } else {
            false
        }
    }
}

impl PartialEq<&Value> for Object {
    fn eq(&self, other: &&Value) -> bool {
        if let Some(value) = self.as_value() {
            *value == **other
        } else {
            false
        }
    }
}

impl serde::Serialize for Object {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Value(v) => v.serialize(serializer),
            Self::Routine(v) => v.name().serialize(serializer),
            Self::Func(v) => v.name().serialize(serializer),
        }
    }
}
