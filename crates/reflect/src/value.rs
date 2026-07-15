use crate::ToType;

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Number(crate::Number),
    Str(crate::Str),
    Map(crate::Map),
    Dynamic(crate::Dynamic),
    Null,
    Undefined,
}

impl Value {
    pub const UNDEFINED: Value = Value::Undefined;

    pub fn as_ref(&self) -> ValueRef<'_> {
        match self {
            Self::Bool(v) => ValueRef::Bool(*v),
            Self::Number(v) => ValueRef::Number(*v),
            Self::Str(v) => ValueRef::Str(v.as_ref()),
            Self::Map(v) => ValueRef::Map(v),
            Self::Dynamic(v) => ValueRef::Dynamic(v.as_ref()),
            Self::Null => ValueRef::Null,
            Self::Undefined => ValueRef::Undefined,
        }
    }

    pub fn cast<T: TryFrom<Value>>(self) -> Option<T> {
        T::try_from(self).ok()
    }

    pub fn is<T: TryFrom<Value>>(&self) -> bool {
        T::try_from(self.clone()).is_ok()
    }

    pub fn is_truthy(&self) -> bool {
        self.as_ref().is_truthy()
    }

    pub fn field(&self, name: &str) -> Result<Value, String> {
        let object = self
            .as_dynamic()
            .and_then(|d| d.as_object())
            .ok_or_else(|| format!("'{}' is not an object", self.to_type()))?;
        Ok(object.field(name))
    }

    pub fn field_by_ref(&self, name: &str) -> Result<ValueRef<'_>, String> {
        let object = self
            .as_dynamic()
            .and_then(|d| d.as_object())
            .ok_or_else(|| format!("'{}' is not an object", self.to_type()))?;
        Ok(object.field_by_ref(name))
    }

    pub fn index(&self, i: usize) -> Result<Value, String> {
        let seq = self
            .as_dynamic()
            .and_then(|d| d.as_sequence())
            .ok_or_else(|| format!("'{}' is not a sequence", self.to_type()))?;

        if i >= seq.len() {
            return Err(format!("index {} out of bounds", i));
        }

        Ok(seq.index(i))
    }

    pub fn index_by_ref(&self, i: usize) -> Result<ValueRef<'_>, String> {
        let seq = self
            .as_dynamic()
            .and_then(|d| d.as_sequence())
            .ok_or_else(|| format!("'{}' is not a sequence", self.to_type()))?;

        if i >= seq.len() {
            return Err(format!("index {} out of bounds", i));
        }

        Ok(seq.index_by_ref(i))
    }

    pub fn call(&self, name: &str, args: &[ValueRef]) -> Result<Value, String> {
        let object = self
            .as_dynamic()
            .and_then(|d| d.as_object())
            .ok_or_else(|| format!("'{}' is not an object", self.to_type()))?;
        crate::Object::call(object, name, args)
    }

    pub fn to_type(&self) -> crate::Type {
        match self {
            Self::Bool(v) => v.to_type(),
            Self::Number(v) => v.to_type(),
            Self::Str(v) => v.to_type(),
            Self::Map(v) => v.to_type(),
            Self::Dynamic(v) => v.to_type(),
            Self::Null | Self::Undefined => crate::Type::Void,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Map(v) => v.len(),
            Self::Str(v) => v.len(),
            Self::Dynamic(v) if v.is_sequence() => v.len(),
            _ => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }
    pub fn is_str(&self) -> bool {
        matches!(self, Self::Str(_))
    }
    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }
    pub fn is_dynamic(&self) -> bool {
        matches!(self, Self::Dynamic(_))
    }
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined)
    }

    pub fn as_bool(&self) -> Option<&bool> {
        match self {
            Self::Bool(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<&crate::Number> {
        match self {
            Self::Number(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&crate::Str> {
        match self {
            Self::Str(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_dynamic(&self) -> Option<&crate::Dynamic> {
        match self {
            Self::Dynamic(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&crate::Map> {
        match self {
            Self::Map(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_bool(&self) -> Option<bool> {
        self.as_bool().copied()
    }

    pub fn to_number(&self) -> Option<crate::Number> {
        self.as_number().copied()
    }

    pub fn to_str(&self) -> Option<crate::Str> {
        self.as_str().cloned()
    }

    pub fn to_dynamic(&self) -> Option<crate::Dynamic> {
        self.as_dynamic().cloned()
    }

    pub fn to_map(&self) -> Option<crate::Map> {
        self.as_map().cloned()
    }
}

impl AsRef<Value> for Value {
    fn as_ref(&self) -> &Value {
        self
    }
}

impl<'a> From<ValueRef<'a>> for Value {
    fn from(value: ValueRef<'a>) -> Self {
        value.to_owned()
    }
}

impl<'a> From<&ValueRef<'a>> for Value {
    fn from(value: &ValueRef<'a>) -> Self {
        value.to_owned()
    }
}

impl crate::TypeOf for Value {
    fn type_of() -> crate::Type {
        crate::Type::Any
    }
}

impl crate::ToType for Value {
    fn to_type(&self) -> crate::Type {
        Value::to_type(self)
    }
}

impl crate::ToValue for Value {
    fn to_value_ref(&self) -> crate::ValueRef<'_> {
        self.as_ref()
    }

    fn to_value(&self) -> Value {
        self.clone()
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Bool(v) => other.as_bool() == Some(v),
            Self::Number(v) => other.as_number() == Some(v),
            Self::Str(v) => other.as_str() == Some(v),
            Self::Map(v) => other.as_map() == Some(v),
            Self::Null => other.is_null(),
            Self::Undefined => other.is_undefined(),
            _ => false,
        }
    }
}

impl<'a> PartialEq<ValueRef<'a>> for Value {
    fn eq(&self, other: &ValueRef<'a>) -> bool {
        self.as_ref() == *other
    }
}

impl std::ops::Index<&str> for Value {
    type Output = Self;

    fn index(&self, index: &str) -> &Self::Output {
        match self {
            Self::Map(v) => v.index(&Value::Str(crate::Str::from(index))),
            _ => &Value::UNDEFINED,
        }
    }
}

impl std::ops::Index<&Self> for Value {
    type Output = Self;

    fn index(&self, index: &Self) -> &Self::Output {
        match self {
            Self::Map(v) => v.index(index),
            _ => &Value::UNDEFINED,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(v) => write!(f, "{}", v),
            Self::Number(v) => write!(f, "{}", v),
            Self::Str(v) => write!(f, "{}", v),
            Self::Map(v) => write!(f, "{}", v),
            Self::Dynamic(v) => write!(f, "{}", v),
            Self::Null => write!(f, "<null>"),
            Self::Undefined => write!(f, "<undefined>"),
        }
    }
}

impl Eq for Value {}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let ord = match (self, other) {
            (Self::Bool(a), Self::Bool(b)) => a.partial_cmp(b),
            (Self::Number(a), Self::Number(b)) => a.partial_cmp(b),
            (Self::Str(a), Self::Str(b)) => a.0.partial_cmp(&b.0),
            _ => None,
        };

        ord.unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Value {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Bool(v) => v.serialize(s),
            Self::Number(v) => v.serialize(s),
            Self::Str(v) => v.serialize(s),
            Self::Map(v) => v.serialize(s),
            Self::Dynamic(v) => v.serialize(s),
            Self::Null | Self::Undefined => s.serialize_none(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ValueRef<'a> {
    Bool(bool),
    Number(crate::Number),
    Str(&'a str),
    Map(&'a crate::Map),
    Dynamic(crate::DynamicRef<'a>),
    Null,
    Undefined,
}

impl<'a> ValueRef<'a> {
    pub const UNDEFINED: ValueRef<'static> = ValueRef::Undefined;

    pub fn to_owned(&self) -> Value {
        match self {
            Self::Bool(v) => Value::Bool(*v),
            Self::Number(v) => Value::Number(*v),
            Self::Str(v) => Value::Str(crate::Str::from(*v)),
            Self::Map(v) => Value::Map((*v).clone()),
            Self::Null => Value::Null,
            Self::Undefined => Value::Undefined,
            Self::Dynamic(v) => v.materialize(),
        }
    }

    pub fn to_type(&self) -> crate::Type {
        match self {
            Self::Bool(v) => v.to_type(),
            Self::Number(v) => v.to_type(),
            Self::Str(_) => crate::Type::Str(crate::StrType),
            Self::Map(v) => v.to_type(),
            Self::Dynamic(v) => v.to_type(),
            Self::Null | Self::Undefined => crate::Type::Void,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Map(v) => v.len(),
            Self::Str(v) => v.len(),
            Self::Dynamic(v) if v.is_sequence() => v.len(),
            _ => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn cast<T: for<'b> TryFrom<ValueRef<'b>>>(self) -> Option<T> {
        T::try_from(self).ok()
    }

    pub fn is<T: for<'b> TryFrom<ValueRef<'b>>>(&self) -> bool {
        T::try_from(self.clone()).is_ok()
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            Self::Number(v) => v.to_f64() != 0.0,
            Self::Str(v) => !v.is_empty(),
            Self::Map(v) => !v.is_empty(),
            Self::Dynamic(v) if v.is_sequence() => !v.is_empty(),
            Self::Dynamic(_) => true,
            Self::Null | Self::Undefined => false,
        }
    }

    pub fn field(&self, name: &str) -> Result<ValueRef<'a>, String> {
        let object = self
            .as_dynamic()
            .and_then(|d| d.as_object())
            .ok_or_else(|| format!("'{}' is not an object", self.to_type()))?;
        Ok(object.field_by_ref(name))
    }

    pub fn index(&self, i: usize) -> Result<ValueRef<'a>, String> {
        let dynamic = self
            .as_dynamic()
            .ok_or_else(|| format!("'{}' is not a sequence", self.to_type()))?;
        let seq = dynamic
            .as_sequence()
            .ok_or_else(|| format!("'{}' is not a sequence", self.to_type()))?;

        if i >= seq.len() {
            return Err(format!("index {} out of bounds", i));
        }

        Ok(seq.index_by_ref(i))
    }

    pub fn call(&self, name: &str, args: &[ValueRef]) -> Result<Value, String> {
        let object = self
            .as_dynamic()
            .and_then(|d| d.as_object())
            .ok_or_else(|| format!("'{}' is not an object", self.to_type()))?;
        crate::Object::call(object, name, args)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }
    pub fn is_str(&self) -> bool {
        matches!(self, Self::Str(_))
    }
    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }
    pub fn is_dynamic(&self) -> bool {
        matches!(self, Self::Dynamic(_))
    }
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined)
    }

    pub fn as_bool(&self) -> Option<&bool> {
        match self {
            Self::Bool(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<&crate::Number> {
        match self {
            Self::Number(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&'a str> {
        match self {
            Self::Str(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_dynamic(&self) -> Option<crate::DynamicRef<'a>> {
        match self {
            Self::Dynamic(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&'a crate::Map> {
        match self {
            Self::Map(v) => Some(*v),
            _ => None,
        }
    }

    pub fn to_bool(&self) -> Option<bool> {
        self.as_bool().copied()
    }

    pub fn to_number(&self) -> Option<crate::Number> {
        self.as_number().copied()
    }

    pub fn to_str(&self) -> Option<String> {
        self.as_str().map(|v| v.to_string())
    }

    pub fn to_dynamic(&self) -> Option<crate::DynamicRef<'a>> {
        self.as_dynamic()
    }

    pub fn to_map(&self) -> Option<crate::Map> {
        self.as_map().cloned()
    }
}

impl<'a> AsRef<ValueRef<'a>> for ValueRef<'a> {
    fn as_ref(&self) -> &ValueRef<'a> {
        self
    }
}

impl<'a> crate::TypeOf for ValueRef<'a> {
    fn type_of() -> crate::Type {
        crate::Type::Any
    }
}

impl<'a> crate::ToType for ValueRef<'a> {
    fn to_type(&self) -> crate::Type {
        ValueRef::to_type(self)
    }
}

impl<'a> crate::ToValue for ValueRef<'a> {
    fn to_value_ref(&self) -> crate::ValueRef<'_> {
        self.clone()
    }
}

impl<'a> PartialEq for ValueRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Bool(v) => other.as_bool() == Some(v),
            Self::Number(v) => other.as_number() == Some(v),
            Self::Str(v) => other.as_str() == Some(*v),
            Self::Map(v) => other.as_map() == Some(*v),
            Self::Null => other.is_null(),
            Self::Undefined => other.is_undefined(),
            _ => false,
        }
    }
}

impl<'a> PartialEq<Value> for ValueRef<'a> {
    fn eq(&self, other: &Value) -> bool {
        *self == other.as_ref()
    }
}

impl<'a> PartialEq<dyn crate::ToValue> for ValueRef<'a> {
    fn eq(&self, other: &dyn crate::ToValue) -> bool {
        self.eq(&other.to_value())
    }
}

impl<'a> std::fmt::Display for ValueRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(v) => write!(f, "{}", v),
            Self::Number(v) => write!(f, "{}", v),
            Self::Str(v) => write!(f, "{}", v),
            Self::Map(v) => write!(f, "{}", v),
            Self::Dynamic(v) => write!(f, "{}", v),
            Self::Null => write!(f, "<null>"),
            Self::Undefined => write!(f, "<undefined>"),
        }
    }
}

impl<'a> Eq for ValueRef<'a> {}

impl<'a> Ord for ValueRef<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let ord = match (self, other) {
            (Self::Bool(a), Self::Bool(b)) => a.partial_cmp(b),
            (Self::Number(a), Self::Number(b)) => a.partial_cmp(b),
            (Self::Str(a), Self::Str(b)) => a.partial_cmp(b),
            _ => None,
        };

        ord.unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl<'a> PartialOrd for ValueRef<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(feature = "serde")]
impl<'a> serde::Serialize for ValueRef<'a> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Bool(v) => v.serialize(s),
            Self::Number(v) => v.serialize(s),
            Self::Str(v) => v.serialize(s),
            Self::Map(v) => v.serialize(s),
            Self::Dynamic(v) => v.serialize(s),
            Self::Null | Self::Undefined => s.serialize_none(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    pub fn as_returns_none_on_mismatch() {
        assert_eq!(value_of!(true).as_number(), None);
        assert_eq!(value_of!(1_i32).as_str(), None);
        assert_eq!(value_of!("x").as_bool(), None);
        assert!(value_of!(true).as_dynamic().is_none());
    }

    #[test]
    pub fn as_returns_some_on_match() {
        assert!(value_of!(true).as_bool().is_some());
        assert!(value_of!(1_i32).as_number().is_some());
        assert!(value_of!("x").as_str().is_some());
    }

    #[test]
    pub fn map_key_ordering_stays_total() {
        let map = btree_map! {
            "b".to_string() => 2_i32,
            "a".to_string() => 1_i32,
            "c".to_string() => 3_i32
        };
        let value = value_of!(map);

        assert!(value.is_map());
        assert_eq!(value.len(), 3);
        assert_eq!(value["a"], value_of!(1_i32));
        assert_eq!(value["b"], value_of!(2_i32));
        assert_eq!(value["c"], value_of!(3_i32));
    }
}
