/// ## Object
///
/// implemented by types that can reflect their value/type
/// and the values of their individual fields
pub trait Object: std::fmt::Debug + Send + Sync + crate::ToType {
    fn field(&self, name: &crate::FieldName) -> crate::Value<'_>;
}

impl std::fmt::Display for dyn Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ty = self.to_type().to_struct();
        write!(f, "{{")?;

        for field in ty.fields().iter() {
            write!(f, "\n\t{}: {}", field.name(), self.field(field.name()))?;
        }
        write!(f, "\n}}")
    }
}
