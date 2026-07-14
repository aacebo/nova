use std::any::Any;
use std::sync::Arc;

use nova_reflect::{ToValue, Value};

pub trait Valueable: ToValue + Send + Sync + std::fmt::Debug + 'static {
    fn as_any(&self) -> &dyn Any;
}

impl<T: ToValue + Send + Sync + std::fmt::Debug + 'static> Valueable for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub enum Pointer {
    Value(Arc<dyn Valueable>),
    Call(Arc<dyn Call>),
    Namespace(Arc<dyn Namespace>),
    Bound {
        call: Arc<dyn Call>,
        namespace: Arc<dyn Namespace>,
    },
}

impl Pointer {
    pub fn new<T: Valueable>(value: T) -> Self {
        Self::Value(Arc::new(value))
    }

    pub fn callable<T: Call>(value: T) -> Self {
        Self::Call(Arc::new(value))
    }

    pub fn namespace<T: Namespace>(value: T) -> Self {
        Self::Namespace(Arc::new(value))
    }

    pub fn callable_namespace<T: Call + Namespace>(value: T) -> Self {
        let value = Arc::new(value);

        Self::Bound {
            call: value.clone() as Arc<dyn Call>,
            namespace: value as Arc<dyn Namespace>,
        }
    }

    pub fn as_namespace(&self) -> Option<&dyn Namespace> {
        match self {
            Self::Namespace(v) => Some(v.as_ref()),
            Self::Bound { namespace, .. } => Some(namespace.as_ref()),
            _ => None,
        }
    }

    pub fn as_call(&self) -> Option<&dyn Call> {
        match self {
            Self::Call(v) => Some(v.as_ref()),
            Self::Bound { call, .. } => Some(call.as_ref()),
            _ => None,
        }
    }

    pub fn is_callable(&self) -> bool {
        self.as_call().is_some()
    }

    pub fn value(&self) -> Value<'_> {
        match self {
            Self::Value(v) => v.to_value(),
            _ => Value::Undefined,
        }
    }

    fn as_any(&self) -> &dyn Any {
        match self {
            Self::Value(v) => v.as_any(),
            Self::Call(v) => v.as_any(),
            Self::Namespace(v) => v.as_any(),
            Self::Bound { call, .. } => call.as_any(),
        }
    }

    pub fn downcast<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }

    pub fn is<T: 'static>(&self) -> bool {
        self.as_any().is::<T>()
    }

    pub fn is_truthy(&self) -> bool {
        is_truthy(&self.value())
    }

    fn valueable(&self) -> Option<&Arc<dyn Valueable>> {
        match self {
            Self::Value(v) => Some(v),
            _ => None,
        }
    }

    pub fn field(&self, name: &str) -> Option<Pointer> {
        if let Some(namespace) = self.as_namespace() {
            return namespace.member(name);
        }

        let parent = self.valueable()?;
        let value = self.value();
        value.as_dynamic()?.as_object()?;

        Some(Pointer::new(Field {
            parent: parent.clone(),
            name: name.to_string(),
        }))
    }

    pub fn index(&self, i: usize) -> Option<Pointer> {
        let parent = self.valueable()?;
        let value = self.value();
        let seq = value.as_dynamic()?.as_sequence()?;

        if i >= seq.len() {
            return None;
        }

        Some(Pointer::new(Index {
            parent: parent.clone(),
            index: i,
        }))
    }

    pub fn key(&self, key: Value<'static>) -> Option<Pointer> {
        let parent = self.valueable()?;
        let value = self.value();
        value.as_map()?.get(&key)?;

        Some(Pointer::new(Key {
            parent: parent.clone(),
            key,
        }))
    }
}

#[derive(Debug)]
struct Key {
    parent: Arc<dyn Valueable>,
    key: Value<'static>,
}

impl ToValue for Key {
    fn to_value(&self) -> Value<'_> {
        let parent = self.parent.to_value();

        match parent.as_map().and_then(|m| m.get(&self.key)) {
            Some(value) => value.clone(),
            None => Value::Undefined,
        }
    }
}

#[derive(Debug)]
struct Index {
    parent: Arc<dyn Valueable>,
    index: usize,
}

impl ToValue for Index {
    fn to_value(&self) -> Value<'_> {
        let parent = self.parent.to_value();

        match parent.as_dynamic().and_then(|d| d.as_sequence()) {
            Some(seq) => seq.index(self.index),
            None => Value::Undefined,
        }
    }
}

#[derive(Debug)]
struct Field {
    parent: Arc<dyn Valueable>,
    name: String,
}

impl ToValue for Field {
    fn to_value(&self) -> Value<'_> {
        let parent = self.parent.to_value();

        match parent.as_dynamic().and_then(|d| d.as_object()) {
            Some(object) => object.field(&self.name),
            None => Value::Undefined,
        }
    }
}

impl ToValue for Pointer {
    fn to_value(&self) -> Value<'_> {
        self.value()
    }
}

impl nova_reflect::TypeOf for Pointer {
    fn type_of() -> nova_reflect::Type {
        nova_reflect::Type::Any
    }
}

impl nova_reflect::ToType for Pointer {
    fn to_type(&self) -> nova_reflect::Type {
        self.value().to_type()
    }
}

impl std::fmt::Debug for Pointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(v) => v.fmt(f),
            Self::Call(v) => v.fmt(f),
            Self::Namespace(v) => v.fmt(f),
            Self::Bound { call, .. } => call.fmt(f),
        }
    }
}

impl std::fmt::Display for Pointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl PartialEq for Pointer {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl PartialEq<Value<'_>> for Pointer {
    fn eq(&self, other: &Value<'_>) -> bool {
        self.value() == *other
    }
}

impl Eq for Pointer {}

impl Ord for Pointer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value().cmp(&other.value())
    }
}

impl PartialOrd for Pointer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<Value<'_>> for Pointer {
    fn partial_cmp(&self, other: &Value<'_>) -> Option<std::cmp::Ordering> {
        self.value().partial_cmp(other)
    }
}

macro_rules! try_from_pointer {
    ($($ty:ty),* $(,)?) => {
        $(
            impl TryFrom<Pointer> for $ty {
                type Error = crate::Error;

                fn try_from(value: Pointer) -> Result<Self, Self::Error> {
                    <$ty>::try_from(value.value()).map_err(crate::Error::message)
                }
            }

            impl TryFrom<&Pointer> for $ty {
                type Error = crate::Error;

                fn try_from(value: &Pointer) -> Result<Self, Self::Error> {
                    <$ty>::try_from(value.value()).map_err(crate::Error::message)
                }
            }
        )*
    };
}

try_from_pointer!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64);

impl TryFrom<Pointer> for String {
    type Error = crate::Error;

    fn try_from(value: Pointer) -> Result<Self, Self::Error> {
        match value.value().as_str() {
            Some(v) => Ok(v.to_string()),
            None => Err(crate::Error::message("value is not a string")),
        }
    }
}

impl TryFrom<Pointer> for bool {
    type Error = crate::Error;

    fn try_from(value: Pointer) -> Result<Self, Self::Error> {
        Ok(is_truthy(&value.value()))
    }
}

impl serde::Serialize for Pointer {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Pointer {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(PointerVisitor)
    }
}

struct PointerVisitor;

impl<'de> serde::de::Visitor<'de> for PointerVisitor {
    type Value = Pointer;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "any value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
        Ok(Pointer::new(Value::Bool(v)))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
        Ok(Pointer::new(Value::Number(nova_reflect::Number::Int(
            nova_reflect::Int::I64(v),
        ))))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
        Ok(Pointer::new(Value::Number(nova_reflect::Number::Int(
            nova_reflect::Int::U64(v),
        ))))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
        Ok(Pointer::new(Value::Number(nova_reflect::Number::Float(
            nova_reflect::Float::F64(v),
        ))))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
        Ok(Pointer::new(Value::Str(nova_reflect::Str(std::borrow::Cow::Owned(
            v.to_string(),
        )))))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Pointer::new(Value::Null))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Pointer::new(Value::Null))
    }

    fn visit_some<D: serde::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        d.deserialize_any(PointerVisitor)
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut items: Vec<Pointer> = Vec::new();

        while let Some(item) = seq.next_element::<Pointer>()? {
            items.push(item);
        }

        Ok(Pointer::new(items))
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        let mut entries: std::collections::BTreeMap<Pointer, Pointer> = std::collections::BTreeMap::new();

        while let Some((key, value)) = access.next_entry::<Pointer, Pointer>()? {
            entries.insert(key, value);
        }

        Ok(Pointer::new(entries))
    }
}

impl From<Value<'static>> for Pointer {
    fn from(value: Value<'static>) -> Self {
        Pointer::new(value)
    }
}

impl From<&Value<'_>> for Pointer {
    fn from(value: &Value<'_>) -> Self {
        Pointer::new(value.clone().into_owned())
    }
}

impl From<&Pointer> for Pointer {
    fn from(value: &Pointer) -> Self {
        value.clone()
    }
}

macro_rules! from_primitive {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for Pointer {
                fn from(value: $ty) -> Self {
                    Pointer::new(Value::from(value))
                }
            }
        )*
    };
}

from_primitive!(bool, i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, String);

impl From<&str> for Pointer {
    fn from(value: &str) -> Self {
        Pointer::new(Value::from(value.to_string()))
    }
}

impl From<&String> for Pointer {
    fn from(value: &String) -> Self {
        Pointer::new(Value::from(value.clone()))
    }
}

impl<T> From<Vec<T>> for Pointer
where
    T: Into<Pointer>,
{
    fn from(value: Vec<T>) -> Self {
        Pointer::new(value.into_iter().map(Into::into).collect::<Vec<Pointer>>())
    }
}

pub trait Call: Send + Sync + std::fmt::Debug + 'static {
    fn call(&self, args: &crate::Args) -> Result<Pointer, crate::Error>;

    fn as_any(&self) -> &dyn Any;
}

pub trait Namespace: Send + Sync + std::fmt::Debug + 'static {
    fn member(&self, name: &str) -> Option<Pointer>;

    fn members(&self) -> Vec<String> {
        Vec::new()
    }

    fn as_any(&self) -> &dyn Any;
}

pub fn is_truthy(value: &Value<'_>) -> bool {
    match value {
        Value::Bool(v) => *v,
        Value::Number(v) => v.to_f64() != 0.0,
        Value::Str(v) => !v.is_empty(),
        Value::Map(v) => !v.is_empty(),
        Value::Ref(v) => is_truthy(v.value),
        Value::Mut(v) => is_truthy(v.value),
        Value::Dynamic(v) if v.is_sequence() => !v.is_empty(),
        Value::Dynamic(_) => true,
        Value::Null | Value::Undefined => false,
    }
}
