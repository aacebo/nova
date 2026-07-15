mod callable;
mod object;
mod sequence;

use std::sync::Arc;

pub use callable::*;
pub use object::*;
pub use sequence::*;

#[derive(Debug, Clone)]
pub enum Dynamic {
    Object(Arc<dyn Object>),
    Sequence(Arc<dyn Sequence>),
    Callable(Arc<dyn Callable>),
    Bound {
        object: Arc<dyn Object>,
        callable: Arc<dyn Callable>,
    },
}

impl Dynamic {
    pub fn from_object<T: Object + 'static>(value: Arc<T>) -> Self {
        Self::Object(value)
    }

    pub fn from_sequence<T: Sequence + 'static>(value: Arc<T>) -> Self {
        Self::Sequence(value)
    }

    pub fn from_callable<T: Callable + 'static>(value: Arc<T>) -> Self {
        Self::Callable(value)
    }

    pub fn from_bound<T: Object + Callable + 'static>(value: Arc<T>) -> Self {
        Self::Bound {
            object: value.clone(),
            callable: value,
        }
    }

    pub fn as_ref(&self) -> DynamicRef<'_> {
        match self {
            Self::Object(v) => DynamicRef::Object(v.as_ref()),
            Self::Sequence(v) => DynamicRef::Sequence(v.as_ref()),
            Self::Callable(v) => DynamicRef::Callable(v.as_ref()),
            Self::Bound { object, callable } => DynamicRef::Bound {
                object: object.as_ref(),
                callable: callable.as_ref(),
            },
        }
    }

    pub fn to_type(&self) -> crate::Type {
        match self {
            Self::Object(v) => v.to_type(),
            Self::Sequence(v) => v.to_type(),
            Self::Callable(v) => v.to_type(),
            Self::Bound { object, .. } => object.to_type(),
        }
    }

    pub fn call(&self, args: &[crate::ValueRef]) -> Result<crate::Value, String> {
        match self {
            Self::Callable(v) => v.call(args),
            Self::Bound { callable, .. } => callable.call(args),
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
        matches!(self, Self::Object(_) | Self::Bound { .. })
    }

    pub fn is_sequence(&self) -> bool {
        matches!(self, Self::Sequence(_))
    }

    pub fn is_callable(&self) -> bool {
        matches!(self, Self::Callable(_) | Self::Bound { .. })
    }

    pub fn as_object(&self) -> Option<&dyn Object> {
        match self {
            Self::Object(v) => Some(v.as_ref()),
            Self::Bound { object, .. } => Some(object.as_ref()),
            _ => None,
        }
    }

    pub fn as_sequence(&self) -> Option<&dyn Sequence> {
        match self {
            Self::Sequence(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    pub fn as_callable(&self) -> Option<&dyn Callable> {
        match self {
            Self::Callable(v) => Some(v.as_ref()),
            Self::Bound { callable, .. } => Some(callable.as_ref()),
            _ => None,
        }
    }
}

impl crate::TypeOf for Dynamic {
    fn type_of() -> crate::Type {
        crate::Type::Any
    }
}

impl crate::ToType for Dynamic {
    fn to_type(&self) -> crate::Type {
        match self {
            Self::Object(v) => v.to_type(),
            Self::Sequence(v) => v.to_type(),
            Self::Callable(v) => v.to_type(),
            Self::Bound { object, .. } => object.to_type(),
        }
    }
}

impl crate::ToValue for Dynamic {
    fn to_value_ref(&self) -> crate::ValueRef<'_> {
        crate::ValueRef::Dynamic(self.as_ref())
    }
}

impl Object for Dynamic {
    fn field_by_ref(&self, name: &str) -> crate::ValueRef<'_> {
        match self {
            Self::Object(v) => v.field_by_ref(name),
            Self::Bound { object, .. } => object.field_by_ref(name),
            _ => crate::ValueRef::Undefined,
        }
    }

    fn field(&self, name: &str) -> crate::Value {
        match self {
            Self::Object(v) => v.field(name),
            Self::Bound { object, .. } => object.field(name),
            _ => crate::Value::Undefined,
        }
    }

    fn call(&self, name: &str, args: &[crate::ValueRef]) -> Result<crate::Value, String> {
        match self {
            Self::Object(v) => v.call(name, args),
            Self::Bound { object, .. } => object.call(name, args),
            _ => Err(format!("no method '{}'", name)),
        }
    }
}

impl Sequence for Dynamic {
    fn len(&self) -> usize {
        match self {
            Self::Sequence(v) => v.len(),
            _ => 0,
        }
    }

    fn index_by_ref(&self, i: usize) -> crate::ValueRef<'_> {
        match self {
            Self::Sequence(v) => v.index_by_ref(i),
            _ => crate::ValueRef::Undefined,
        }
    }

    fn index(&self, i: usize) -> crate::Value {
        match self {
            Self::Sequence(v) => v.index(i),
            _ => crate::Value::Undefined,
        }
    }
}

impl Callable for Dynamic {
    fn call(&self, args: &[crate::ValueRef]) -> Result<crate::Value, String> {
        match self {
            Self::Callable(v) => v.call(args),
            Self::Bound { callable, .. } => callable.call(args),
            v => Err(format!("'{}' is not callable", v.to_type())),
        }
    }
}

impl std::fmt::Display for Dynamic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ty = self.to_type();
        write!(f, "<{}>", ty)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Dynamic {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_ref().serialize(serializer)
    }
}

#[derive(Debug, Clone)]
pub enum DynamicRef<'a> {
    Object(&'a (dyn Object + 'a)),
    Sequence(&'a (dyn Sequence + 'a)),
    Callable(&'a (dyn Callable + 'a)),
    Bound {
        object: &'a (dyn Object + 'a),
        callable: &'a (dyn Callable + 'a),
    },
}

impl<'a> DynamicRef<'a> {
    pub fn from_object<T: Object + 'a>(value: &'a T) -> Self {
        Self::Object(value)
    }

    pub fn from_sequence<T: Sequence + 'a>(value: &'a T) -> Self {
        Self::Sequence(value)
    }

    pub fn from_callable<T: Callable + 'a>(value: &'a T) -> Self {
        Self::Callable(value)
    }

    pub fn from_bound<T: Object + Callable + 'a>(value: &'a T) -> Self {
        Self::Bound {
            object: value,
            callable: value,
        }
    }

    pub fn to_type(&self) -> crate::Type {
        match self {
            Self::Object(v) => v.to_type(),
            Self::Sequence(v) => v.to_type(),
            Self::Callable(v) => v.to_type(),
            Self::Bound { object, .. } => object.to_type(),
        }
    }

    pub fn call(&self, args: &[crate::ValueRef]) -> Result<crate::Value, String> {
        match self {
            Self::Callable(v) => v.call(args),
            Self::Bound { callable, .. } => callable.call(args),
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
        matches!(self, Self::Object(_) | Self::Bound { .. })
    }

    pub fn is_sequence(&self) -> bool {
        matches!(self, Self::Sequence(_))
    }

    pub fn is_callable(&self) -> bool {
        matches!(self, Self::Callable(_) | Self::Bound { .. })
    }

    pub fn materialize(&self) -> crate::Value {
        let ty = crate::MapType::new(crate::Type::Any, crate::Type::Any, crate::Type::Any);

        match self {
            Self::Callable(_) => crate::Value::Undefined,
            Self::Object(v) | Self::Bound { object: v, .. } => {
                if let Some(entries) = v.entries() {
                    return crate::Value::Map(entries);
                }

                let mut map = crate::Map::new(&ty);

                if let Some(st) = v.to_type().to_struct() {
                    for field in st.fields().iter() {
                        let name = field.name().to_string();
                        map.insert(crate::Value::Str(crate::Str::from(name.as_str())), v.field(&name));
                    }
                }

                crate::Value::Map(map)
            }
            Self::Sequence(v) => {
                let data: Vec<crate::Value> = v.iter().collect();
                crate::Value::Dynamic(Dynamic::from_sequence(Arc::new(data)))
            }
        }
    }

    pub fn as_object(&self) -> Option<&'a (dyn Object + 'a)> {
        match self {
            Self::Object(v) => Some(*v),
            Self::Bound { object, .. } => Some(*object),
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
            Self::Bound { callable, .. } => Some(*callable),
            _ => None,
        }
    }
}

impl<'a> crate::TypeOf for DynamicRef<'a> {
    fn type_of() -> crate::Type {
        crate::Type::Any
    }
}

impl<'a> crate::ToType for DynamicRef<'a> {
    fn to_type(&self) -> crate::Type {
        match self {
            Self::Object(v) => v.to_type(),
            Self::Sequence(v) => v.to_type(),
            Self::Callable(v) => v.to_type(),
            Self::Bound { object, .. } => object.to_type(),
        }
    }
}

impl<'a> crate::ToValue for DynamicRef<'a> {
    fn to_value_ref(&self) -> crate::ValueRef<'_> {
        crate::ValueRef::Dynamic(self.clone())
    }
}

impl<'a> Object for DynamicRef<'a> {
    fn field_by_ref(&self, name: &str) -> crate::ValueRef<'_> {
        match self {
            Self::Object(v) => v.field_by_ref(name),
            Self::Bound { object, .. } => object.field_by_ref(name),
            _ => crate::ValueRef::Undefined,
        }
    }

    fn field(&self, name: &str) -> crate::Value {
        match self {
            Self::Object(v) => v.field(name),
            Self::Bound { object, .. } => object.field(name),
            _ => crate::Value::Undefined,
        }
    }

    fn call(&self, name: &str, args: &[crate::ValueRef]) -> Result<crate::Value, String> {
        match self {
            Self::Object(v) => v.call(name, args),
            Self::Bound { object, .. } => object.call(name, args),
            _ => Err(format!("no method '{}'", name)),
        }
    }
}

impl<'a> Sequence for DynamicRef<'a> {
    fn len(&self) -> usize {
        match self {
            Self::Sequence(v) => v.len(),
            _ => 0,
        }
    }

    fn index_by_ref(&self, i: usize) -> crate::ValueRef<'_> {
        match self {
            Self::Sequence(v) => v.index_by_ref(i),
            _ => crate::ValueRef::Undefined,
        }
    }

    fn index(&self, i: usize) -> crate::Value {
        match self {
            Self::Sequence(v) => v.index(i),
            _ => crate::Value::Undefined,
        }
    }
}

impl<'a> Callable for DynamicRef<'a> {
    fn call(&self, args: &[crate::ValueRef]) -> Result<crate::Value, String> {
        match self {
            Self::Callable(v) => v.call(args),
            Self::Bound { callable, .. } => callable.call(args),
            v => Err(format!("'{}' is not callable", v.to_type())),
        }
    }
}

impl<'a> std::fmt::Display for DynamicRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ty = self.to_type();
        write!(f, "<{}>", ty)
    }
}

#[cfg(feature = "serde")]
impl<'a> serde::Serialize for DynamicRef<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        match self {
            Self::Callable(_) => serializer.serialize_none(),
            Self::Object(v) | Self::Bound { object: v, .. } if v.entries_by_ref().is_some() => {
                let entries = v.entries_by_ref().unwrap();
                let mut ser = serializer.serialize_map(Some(entries.len()))?;

                for (k, val) in &entries {
                    ser.serialize_entry(&k.to_string(), val)?;
                }

                ser.end()
            }
            Self::Object(v) | Self::Bound { object: v, .. } => {
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
