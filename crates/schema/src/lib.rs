mod bool;
pub mod error;
mod string;

pub use bool::*;
pub use error::Error;
pub use string::*;

pub trait Validate {
    fn validate(&self, value: &reflect::Value) -> Result<(), Error>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Schema {
    String(StringSchema),
}
