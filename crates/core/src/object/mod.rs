mod annotation;
mod artifact;
mod function;
mod namespace;
mod var;

pub use annotation::*;
pub use artifact::*;
pub use function::*;
pub use namespace::*;
pub use var::*;

use crate::{Action, Call, Predicate};

#[derive(Clone)]
pub enum Object {
    Func(Function),
    Namespace(Namespace),
    Artifact(Artifact),
    Annotation(Annotation),
    Var(Var),
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

    pub fn namespace(namespace: Namespace) -> Self {
        Self::Namespace(namespace)
    }

    pub fn is_func(&self) -> bool {
        matches!(self, Self::Func(_))
    }

    pub fn is_namespace(&self) -> bool {
        matches!(self, Self::Namespace(_))
    }

    pub fn is_artifact(&self) -> bool {
        matches!(self, Self::Artifact(_))
    }

    pub fn is_annotation(&self) -> bool {
        matches!(self, Self::Annotation(_))
    }

    pub fn is_var(&self) -> bool {
        matches!(self, Self::Var(_))
    }

    pub fn as_func(&self) -> Option<&Function> {
        match self {
            Self::Func(v) => Some(v),
            Self::Namespace(v) => Some(v.entrypoint()),
            _ => None,
        }
    }

    pub fn as_namespace(&self) -> Option<&Namespace> {
        match self {
            Self::Namespace(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_artifact(&self) -> Option<&Artifact> {
        match self {
            Self::Artifact(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_artifact_mut(&mut self) -> Option<&mut Artifact> {
        match self {
            Self::Artifact(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_annotation(&self) -> Option<&Annotation> {
        match self {
            Self::Annotation(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_annotation_mut(&mut self) -> Option<&mut Annotation> {
        match self {
            Self::Annotation(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_var(&self) -> Option<&Var> {
        match self {
            Self::Var(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_var_mut(&mut self) -> Option<&mut Var> {
        match self {
            Self::Var(v) => Some(v),
            _ => None,
        }
    }
}

impl From<Function> for Object {
    fn from(value: Function) -> Self {
        Self::Func(value)
    }
}

impl From<Namespace> for Object {
    fn from(value: Namespace) -> Self {
        Self::Namespace(value)
    }
}

impl From<Artifact> for Object {
    fn from(value: Artifact) -> Self {
        Self::Artifact(value)
    }
}

impl From<Annotation> for Object {
    fn from(value: Annotation) -> Self {
        Self::Annotation(value)
    }
}

impl From<Var> for Object {
    fn from(value: Var) -> Self {
        Self::Var(value)
    }
}
