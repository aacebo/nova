mod function;
mod routine;

pub use function::*;
use nova_reflect::Value;
pub use routine::*;

use crate::{Action, Call, Predicate};

#[derive(Debug, Clone)]
pub enum Binding {
    Func(Function),
    Routine(Routine),
    Value(Value),
    Pointer(nova_template::Pointer),
}

impl Binding {
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

    pub fn value(value: impl Into<nova_template::Pointer>) -> Self {
        match value.into() {
            nova_template::Pointer::Value(v) => Self::Value(v),
            other => Self::Pointer(other),
        }
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

impl From<Function> for Binding {
    fn from(value: Function) -> Self {
        Self::Func(value)
    }
}

impl From<Routine> for Binding {
    fn from(value: Routine) -> Self {
        Self::Routine(value)
    }
}

impl<T: Into<Value>> From<T> for Binding {
    fn from(value: T) -> Self {
        Self::Value(value.into())
    }
}

impl Eq for Binding {}

impl PartialEq for Binding {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Func(a), Self::Func(b)) => a.name() == b.name(),
            (Self::Routine(a), Self::Routine(b)) => a.name() == b.name(),
            (Self::Value(a), Self::Value(b)) => *a == *b,
            _ => false,
        }
    }
}

impl PartialEq<nova_template::Pointer> for Binding {
    fn eq(&self, other: &nova_template::Pointer) -> bool {
        match self.as_value() {
            Some(value) => *other == *value,
            None => false,
        }
    }
}

impl PartialEq<&nova_template::Pointer> for Binding {
    fn eq(&self, other: &&nova_template::Pointer) -> bool {
        match self.as_value() {
            Some(value) => **other == *value,
            None => false,
        }
    }
}

impl PartialEq<nova_reflect::ValueRef<'_>> for Binding {
    fn eq(&self, other: &nova_reflect::ValueRef<'_>) -> bool {
        match self.as_value() {
            Some(value) => value.as_ref() == *other,
            None => false,
        }
    }
}

impl PartialEq<nova_reflect::Value> for Binding {
    fn eq(&self, other: &nova_reflect::Value) -> bool {
        match self.as_value() {
            Some(value) => value == other,
            None => false,
        }
    }
}

impl serde::Serialize for Binding {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Value(v) => v.serialize(serializer),
            Self::Pointer(v) => v.serialize(serializer),
            Self::Routine(v) => v.name().serialize(serializer),
            Self::Func(v) => v.name().serialize(serializer),
        }
    }
}
