mod callable;
mod object;
mod sequence;

pub use callable::*;
pub use object::*;
pub use sequence::*;

#[derive(Debug, Clone)]
pub enum Dynamic<'a> {
    Object(&'a (dyn Object + 'a)),
    Sequence(&'a (dyn Sequence + 'a)),
    Callable(&'a (dyn Callable + 'a)),
}

impl<'a> Dynamic<'a> {
    pub fn from_object<T: Object + 'a>(value: &'a T) -> Self {
        Self::Object(value)
    }

    pub fn from_sequence<T: Sequence + 'a>(value: &'a T) -> Self {
        Self::Sequence(value)
    }

    pub fn from_callable<T: Callable + 'a>(value: &'a T) -> Self {
        Self::Callable(value)
    }

    pub fn to_type(&self) -> crate::Type {
        match self {
            Self::Object(v) => v.to_type(),
            Self::Sequence(v) => v.to_type(),
            Self::Callable(v) => v.to_type(),
        }
    }

    pub fn call(&self, args: &[crate::Value]) -> Result<crate::Value<'_>, String> {
        match self {
            Self::Callable(v) => v.call(args),
            v => Err(format!("'{}' is not callable", v.to_type())),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Sequence(v) => v.len(),
            _ => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    pub fn is_sequence(&self) -> bool {
        matches!(self, Self::Sequence(_))
    }

    pub fn is_callable(&self) -> bool {
        matches!(self, Self::Callable(_))
    }

    pub fn as_object(&self) -> Option<&'a (dyn Object + 'a)> {
        match self {
            Self::Object(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_sequence(&self) -> Option<&'a (dyn Sequence + 'a)> {
        match self {
            Self::Sequence(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_callable(&self) -> Option<&'a (dyn Callable + 'a)> {
        match self {
            Self::Callable(v) => Some(*v),
            _ => None,
        }
    }
}

impl<'a> crate::TypeOf for Dynamic<'a> {
    fn type_of() -> crate::Type {
        crate::Type::Any
    }
}

impl<'a> crate::ToType for Dynamic<'a> {
    fn to_type(&self) -> crate::Type {
        match self {
            Self::Object(v) => v.to_type(),
            Self::Sequence(v) => v.to_type(),
            Self::Callable(v) => v.to_type(),
        }
    }
}

impl<'a> crate::ToValue for Dynamic<'a> {
    fn to_value(&self) -> crate::Value<'_> {
        crate::Value::Dynamic(self.clone())
    }
}

impl<'a> Object for Dynamic<'a> {
    fn field(&self, name: &str) -> crate::Value<'_> {
        match self {
            Self::Object(v) => v.field(name),
            _ => crate::Value::Undefined,
        }
    }

    fn call(&self, name: &str, args: &[crate::Value]) -> Result<crate::Value<'_>, String> {
        match self {
            Self::Object(v) => v.call(name, args),
            _ => Err(format!("no method '{}'", name)),
        }
    }
}

impl<'a> Sequence for Dynamic<'a> {
    fn len(&self) -> usize {
        match self {
            Self::Sequence(v) => v.len(),
            _ => 0,
        }
    }

    fn index(&self, i: usize) -> crate::Value<'_> {
        match self {
            Self::Sequence(v) => v.index(i),
            _ => crate::Value::Undefined,
        }
    }
}

impl<'a> Callable for Dynamic<'a> {
    fn call(&self, args: &[crate::Value]) -> Result<crate::Value<'_>, String> {
        match self {
            Self::Callable(v) => v.call(args),
            v => Err(format!("'{}' is not callable", v.to_type())),
        }
    }
}

impl<'a> std::fmt::Display for Dynamic<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ty = self.to_type();
        write!(f, "<{}>", ty)
    }
}

#[cfg(feature = "serde")]
impl<'a> serde::Serialize for Dynamic<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        match self {
            Self::Callable(_) => serializer.serialize_none(),
            Self::Object(v) => {
                let ty = v.to_type().to_struct();
                let fields = ty.as_ref().map(|t| t.fields());
                let len = fields.map(|f| f.len()).unwrap_or(0);
                let mut ser = serializer.serialize_map(Some(len))?;

                if let Some(fields) = fields {
                    for field in fields.iter() {
                        let name = field.name().to_string();
                        ser.serialize_entry(&name, &v.field(&name))?;
                    }
                }

                ser.end()
            }
            Self::Sequence(v) => {
                use serde::ser::SerializeSeq;
                let mut ser = serializer.serialize_seq(Some(v.len()))?;

                for i in 0..v.len() {
                    ser.serialize_element(&v.index(i))?;
                }

                ser.end()
            }
        }
    }
}
