mod annotation;
mod artifact;
mod var;

use std::sync::Arc;

pub use annotation::*;
pub use artifact::*;
pub use var::*;

use crate::{Action, Predicate};

#[derive(Clone)]
pub enum Object {
    Action(Arc<dyn Action>),
    Predicate(Arc<dyn Predicate>),
    Artifact(Artifact),
    Annotation(Annotation),
    Var(Var),
}

impl Object {
    pub fn predicate(predicate: impl Predicate + 'static) -> Self {
        Self::Predicate(Arc::new(predicate))
    }

    pub fn is_action(&self) -> bool {
        matches!(self, Self::Action(_))
    }

    pub fn is_predicate(&self) -> bool {
        matches!(self, Self::Predicate(_))
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

    pub fn as_action(&self) -> Option<&dyn Action> {
        match self {
            Self::Action(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    pub fn as_predicate(&self) -> Option<&dyn Predicate> {
        match self {
            Self::Predicate(v) => Some(v.as_ref()),
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

impl<T: Action + 'static> From<T> for Object {
    fn from(value: T) -> Self {
        Self::Action(Arc::new(value))
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
