use crate::{Args, Predicate, scope};

pub enum Cmp {
    Eq,
    Gt,
    Lt,
}

impl std::fmt::Display for Cmp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eq => write!(f, "="),
            Self::Gt => write!(f, ">"),
            Self::Lt => write!(f, "<"),
        }
    }
}

pub struct Compare {
    a: String,
    b: String,
    style: Cmp,
}

impl Compare {
    pub fn new(a: impl Into<String>, b: impl Into<String>, style: Cmp) -> Self {
        Self {
            a: a.into(),
            b: b.into(),
            style,
        }
    }
}

impl Predicate for Compare {
    fn invoke(&self, _args: &Args) -> Result<bool, Box<dyn std::error::Error>> {
        let scope = scope();
        let a = scope.eval(&self.a)?;
        let b = scope.eval(&self.b)?;

        Ok(match self.style {
            Cmp::Eq => a == b,
            Cmp::Gt => a > b,
            Cmp::Lt => a < b,
        })
    }
}
