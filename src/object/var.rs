use crate::Value;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Var {
    pub name: String,
    pub value: Value,
}

impl Var {
    pub fn new(name: impl Into<String>, value: impl Into<Value>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

impl std::fmt::Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
