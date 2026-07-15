use std::sync::Arc;

use nova_reflect::{Dynamic, ToValue, Value, ValueRef};

use crate::Context;

pub trait Call: Send + Sync + std::fmt::Debug + 'static {
    fn call(&self, ctx: &dyn Context) -> Result<Binding, crate::Error>;
}

pub trait Namespace: Send + Sync + std::fmt::Debug + 'static {
    fn member(&self, name: &str) -> Option<Binding>;

    fn members(&self) -> Vec<String> {
        Vec::new()
    }
}

#[derive(Clone)]
pub enum Binding {
    Value(Value),
    Call(Arc<dyn Call>),
    Namespace(Arc<dyn Namespace>),
    Bound {
        call: Arc<dyn Call>,
        namespace: Arc<dyn Namespace>,
    },
}

impl Binding {
    pub fn new<T: ToValue>(value: T) -> Self {
        Self::Value(value.to_value())
    }

    pub fn value_of(value: Value) -> Self {
        Self::Value(value)
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

    pub fn value(&self) -> ValueRef<'_> {
        match self {
            Self::Value(v) => v.as_ref(),
            _ => ValueRef::Undefined,
        }
    }

    pub fn as_value(&self) -> Option<&Value> {
        match self {
            Self::Value(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_value_mut(&mut self) -> Option<&mut Value> {
        match self {
            Self::Value(v) => Some(v),
            _ => None,
        }
    }

    pub fn into_value(self) -> Value {
        match self {
            Self::Value(v) => v,
            _ => Value::Undefined,
        }
    }

    pub fn to_dynamic_value(&self) -> Value {
        match self {
            Self::Value(v) => v.clone(),
            Self::Call(_) => Value::Dynamic(Dynamic::from_callable(Arc::new(self.clone()))),
            Self::Namespace(_) => Value::Dynamic(Dynamic::from_object(Arc::new(self.clone()))),
            Self::Bound { .. } => Value::Dynamic(Dynamic::from_bound(Arc::new(self.clone()))),
        }
    }

    pub fn cast<T: TryFrom<Self>>(self) -> Option<T> {
        T::try_from(self).ok()
    }

    pub fn is<T: TryFrom<Self>>(&self) -> bool {
        T::try_from(self.clone()).is_ok()
    }

    pub fn is_truthy(&self) -> bool {
        self.value().is_truthy()
    }

    pub fn field(&self, name: &str) -> Option<Self> {
        if let Some(namespace) = self.as_namespace() {
            return namespace.member(name);
        }

        let value = self.value();
        let object = value.as_dynamic()?.as_object()?;
        Some(Self::Value(object.field(name).to_owned()))
    }

    pub fn index(&self, i: usize) -> Option<Self> {
        let value = self.value();
        let dynamic = value.as_dynamic()?;
        let seq = dynamic.as_sequence()?;

        if i >= seq.len() {
            return None;
        }

        Some(Self::Value(seq.index(i).to_owned()))
    }

    pub fn key(&self, key: Value) -> Option<Self> {
        let value = self.value();
        let entry = value.as_map()?.get(&key)?.clone();
        Some(Self::Value(entry))
    }
}

impl ToValue for Binding {
    fn to_value_ref(&self) -> ValueRef<'_> {
        self.value()
    }

    fn to_value(&self) -> Value {
        match self {
            Self::Value(v) => v.clone(),
            _ => Value::Undefined,
        }
    }
}

impl nova_reflect::TypeOf for Binding {
    fn type_of() -> nova_reflect::Type {
        nova_reflect::Type::Any
    }
}

impl nova_reflect::ToType for Binding {
    fn to_type(&self) -> nova_reflect::Type {
        self.value().to_type()
    }
}

impl nova_reflect::Object for Binding {
    fn field_by_ref(&self, name: &str) -> ValueRef<'_> {
        match self {
            Self::Value(v) => match v.as_dynamic().and_then(|d| d.as_object()) {
                Some(object) => object.field_by_ref(name),
                None => ValueRef::Undefined,
            },
            _ => ValueRef::Undefined,
        }
    }

    fn field(&self, name: &str) -> Value {
        match Self::field(self, name) {
            Some(member) => member.to_dynamic_value(),
            None => Value::Undefined,
        }
    }
}

impl nova_reflect::Sequence for Binding {
    fn len(&self) -> usize {
        match self.value().as_dynamic() {
            Some(dynamic) => dynamic.len(),
            None => 0,
        }
    }

    fn index_by_ref(&self, i: usize) -> ValueRef<'_> {
        match self {
            Self::Value(v) => match v.as_dynamic().and_then(|d| d.as_sequence()) {
                Some(seq) if i < seq.len() => seq.index_by_ref(i),
                _ => ValueRef::Undefined,
            },
            _ => ValueRef::Undefined,
        }
    }

    fn index(&self, i: usize) -> Value {
        match Binding::index(self, i) {
            Some(item) => item.into_value(),
            None => Value::Undefined,
        }
    }
}

impl nova_reflect::Callable for Binding {
    fn call(&self, _args: &[ValueRef]) -> Result<Value, String> {
        match self.as_call() {
            Some(_) => Err("binding requires an execution context to be called".to_string()),
            None => Err(format!("'{}' is not callable", self.value().to_type())),
        }
    }
}

impl std::fmt::Debug for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(v) => v.fmt(f),
            Self::Call(v) => v.fmt(f),
            Self::Namespace(v) => v.fmt(f),
            Self::Bound { call, .. } => call.fmt(f),
        }
    }
}

impl std::fmt::Display for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl PartialEq for Binding {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl PartialEq<ValueRef<'_>> for Binding {
    fn eq(&self, other: &ValueRef<'_>) -> bool {
        self.value() == *other
    }
}

impl PartialEq<Value> for Binding {
    fn eq(&self, other: &Value) -> bool {
        self.value() == other.as_ref()
    }
}

impl Eq for Binding {}

impl Ord for Binding {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value().cmp(&other.value())
    }
}

impl PartialOrd for Binding {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<ValueRef<'_>> for Binding {
    fn partial_cmp(&self, other: &ValueRef<'_>) -> Option<std::cmp::Ordering> {
        self.value().partial_cmp(other)
    }
}

impl PartialOrd<Value> for Binding {
    fn partial_cmp(&self, other: &Value) -> Option<std::cmp::Ordering> {
        self.value().partial_cmp(&other.as_ref())
    }
}

macro_rules! try_from_binding {
    ($($ty:ty),* $(,)?) => {
        $(
            impl TryFrom<Binding> for $ty {
                type Error = crate::Error;

                fn try_from(value: Binding) -> Result<Self, Self::Error> {
                    <$ty>::try_from(value.into_value()).map_err(crate::Error::message)
                }
            }

            impl TryFrom<&Binding> for $ty {
                type Error = crate::Error;

                fn try_from(value: &Binding) -> Result<Self, Self::Error> {
                    <$ty>::try_from(value.value()).map_err(crate::Error::message)
                }
            }
        )*
    };
}

try_from_binding!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64);

impl TryFrom<Binding> for String {
    type Error = crate::Error;

    fn try_from(value: Binding) -> Result<Self, Self::Error> {
        match value.value().as_str() {
            Some(v) => Ok(v.to_string()),
            None => Err(crate::Error::message("value is not a string")),
        }
    }
}

impl TryFrom<Binding> for bool {
    type Error = crate::Error;

    fn try_from(value: Binding) -> Result<Self, Self::Error> {
        Ok(value.value().is_truthy())
    }
}

impl serde::Serialize for Binding {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Binding {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(BindingVisitor)
    }
}

struct BindingVisitor;

impl<'de> serde::de::Visitor<'de> for BindingVisitor {
    type Value = Binding;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "any value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
        Ok(Binding::Value(Value::Bool(v)))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
        Ok(Binding::Value(Value::Number(nova_reflect::Number::Int(
            nova_reflect::Int::I64(v),
        ))))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
        Ok(Binding::Value(Value::Number(nova_reflect::Number::Int(
            nova_reflect::Int::U64(v),
        ))))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
        Ok(Binding::Value(Value::Number(nova_reflect::Number::Float(
            nova_reflect::Float::F64(v),
        ))))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
        Ok(Binding::Value(Value::Str(nova_reflect::Str::from(v))))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Binding::Value(Value::Null))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Binding::Value(Value::Null))
    }

    fn visit_some<D: serde::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        d.deserialize_any(BindingVisitor)
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut items: Vec<Value> = Vec::new();

        while let Some(item) = seq.next_element::<Binding>()? {
            items.push(item.into_value());
        }

        Ok(Binding::Value(nova_reflect::value_of!(items)))
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        let ty = nova_reflect::MapType::new(nova_reflect::Type::Any, nova_reflect::Type::Any, nova_reflect::Type::Any);
        let mut map = nova_reflect::Map::new(&ty);

        while let Some((key, value)) = access.next_entry::<Binding, Binding>()? {
            map.insert(key.into_value(), value.into_value());
        }

        Ok(Binding::Value(Value::Map(map)))
    }
}

impl From<Value> for Binding {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

impl From<&Value> for Binding {
    fn from(value: &Value) -> Self {
        Self::Value(value.clone())
    }
}

impl From<ValueRef<'_>> for Binding {
    fn from(value: ValueRef<'_>) -> Self {
        Self::Value(value.to_owned())
    }
}

impl From<&ValueRef<'_>> for Binding {
    fn from(value: &ValueRef<'_>) -> Self {
        Self::Value(value.to_owned())
    }
}

impl From<&Self> for Binding {
    fn from(value: &Self) -> Self {
        value.clone()
    }
}

macro_rules! from_primitive {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for Binding {
                fn from(value: $ty) -> Self {
                    Binding::Value(Value::from(value))
                }
            }
        )*
    };
}

from_primitive!(bool, i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, String);

impl From<&str> for Binding {
    fn from(value: &str) -> Self {
        Self::Value(Value::from(value))
    }
}

impl From<&String> for Binding {
    fn from(value: &String) -> Self {
        Self::Value(Value::from(value.clone()))
    }
}

impl<T> From<Vec<T>> for Binding
where
    T: Into<Self>,
{
    fn from(value: Vec<T>) -> Self {
        let items: Vec<Value> = value.into_iter().map(|v| v.into().into_value()).collect();
        Self::Value(nova_reflect::value_of!(items))
    }
}

impl Call for Binding {
    fn call(&self, ctx: &dyn Context) -> Result<Binding, crate::Error> {
        if let Some(v) = self.as_call() {
            v.call(ctx)
        } else {
            Err(crate::Error::message(format!("\"{}\" is not callable", ctx.name())))
        }
    }
}
