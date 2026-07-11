/// ## Dyn
///
/// implemented by types that can reflect their value/type.
pub trait Dyn: std::fmt::Debug + Send + Sync + crate::ToType {}
