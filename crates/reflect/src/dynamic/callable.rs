/// ## Callable
///
/// implemented by types that can reflect their value/type
/// and be invoked with arguments.
pub trait Callable: std::fmt::Debug + Send + Sync + crate::ToType {
    fn call(&self, args: &[crate::ValueRef]) -> Result<crate::Value, String>;
}
