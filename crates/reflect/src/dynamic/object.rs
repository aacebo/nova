/// ## Object
///
/// implemented by types that can reflect their value/type,
/// the values of their individual fields (`field`, `Undefined`
/// if absent), and invoke their methods (`call`).
pub trait Object: std::fmt::Debug + Send + Sync + crate::ToType {
    fn field(&self, name: &str) -> crate::Value<'_>;

    fn call(&self, name: &str, _args: &[crate::Value]) -> Result<crate::Value<'_>, String> {
        Err(format!("no method '{}'", name))
    }
}

impl std::fmt::Display for dyn Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(ty) = self.to_type().to_struct() else {
            return write!(f, "{{}}");
        };
        write!(f, "{{")?;

        for field in ty.fields().iter() {
            write!(f, "\n\t{}: {}", field.name(), self.field(&field.name().to_string()))?;
        }
        write!(f, "\n}}")
    }
}

/// ## Methods
///
/// bridges reflected methods into [`Object::call`](crate::Object::call).
/// `#[nova_reflect::reflect]` on an `impl` block generates an inherent
/// `call_method` that shadows this blanket default (inherent methods take
/// priority over trait methods), so a type without a reflected `impl` simply
/// has no callable methods.
pub trait Methods {
    fn call_method(&self, name: &str, _args: &[crate::Value]) -> Result<crate::Value<'static>, String> {
        Err(format!("no method '{}'", name))
    }
}

impl<T> Methods for T {}
