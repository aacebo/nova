use crate::ToType;

#[derive(Debug, Clone)]
pub enum Value<'a> {
    Bool(bool),
    Number(crate::Number),
    Str(crate::Str<'a>),
    Map(crate::Map<'a>),
    Mut(crate::Mut<'a>),
    Ref(crate::Ref<'a>),
    Dynamic(crate::Dynamic<'a>),
    Null,
}

impl<'a> Value<'a> {
    pub fn to_type(&self) -> crate::Type {
        match self {
            Self::Bool(v) => v.to_type(),
            Self::Number(v) => v.to_type(),
            Self::Str(v) => v.to_type(),
            Self::Map(v) => v.to_type(),
            Self::Mut(v) => v.to_type(),
            Self::Ref(v) => v.to_type(),
            Self::Dynamic(v) => v.to_type(),
            Self::Null => panic!("called 'to_type' on '<null>'"),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Map(v) => v.len(),
            Self::Str(v) => v.len(),
            Self::Mut(v) => v.len(),
            Self::Ref(v) => v.len(),
            Self::Dynamic(v) if v.is_sequence() => v.len(),
            v => panic!("called 'len' on '{}'", v.to_type()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Self> {
        match self {
            Self::Mut(v) => v.iter(),
            Self::Ref(v) => v.iter(),
            v => panic!(
                "called 'iter' on '{}'; lazy sequences cannot return std::slice::Iter, use as_dynamic().as_sequence() and index by position",
                v.to_type()
            ),
        }
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }
    pub fn is_mut(&self) -> bool {
        matches!(self, Self::Mut(_))
    }
    pub fn is_ref(&self) -> bool {
        matches!(self, Self::Ref(_))
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

    pub fn as_bool(&self) -> Option<&bool> {
        match self {
            Self::Bool(v) => Some(v),
            Self::Ref(v) => v.as_bool(),
            Self::Mut(v) => v.as_bool(),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<&crate::Number> {
        match self {
            Self::Number(v) => Some(v),
            Self::Ref(v) => v.as_number(),
            Self::Mut(v) => v.as_number(),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&crate::Str<'a>> {
        match self {
            Self::Str(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_dynamic(&self) -> Option<&crate::Dynamic<'a>> {
        match self {
            Self::Dynamic(v) => Some(v),
            Self::Ref(v) => v.as_dynamic(),
            Self::Mut(v) => v.as_dynamic(),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&crate::Map<'a>> {
        match self {
            Self::Map(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            Self::Ref(v) => v.to_bool(),
            Self::Mut(v) => v.to_bool(),
            v => panic!("called 'to_bool' on '{}'", v.to_type()),
        }
    }

    pub fn to_number(&self) -> crate::Number {
        match self {
            Self::Number(v) => *v,
            Self::Ref(v) => v.to_number(),
            Self::Mut(v) => v.to_number(),
            v => panic!("called 'to_number' on '{}'", v.to_type()),
        }
    }

    pub fn to_mut(&self) -> crate::Mut<'a> {
        match self {
            Self::Mut(v) => v.clone(),
            v => panic!("called 'to_mut' on '{}'", v.to_type()),
        }
    }

    pub fn to_ref(&self) -> crate::Ref<'a> {
        match self {
            Self::Ref(v) => v.clone(),
            v => panic!("called 'to_ref' on '{}'", v.to_type()),
        }
    }

    pub fn to_str(&self) -> crate::Str<'a> {
        match self {
            Self::Str(v) => v.clone(),
            v => panic!("called 'to_str' on '{}'", v.to_type()),
        }
    }

    pub fn to_dynamic(&self) -> crate::Dynamic<'a> {
        match self {
            Self::Dynamic(v) => v.clone(),
            v => panic!("called 'to_dynamic' on '{}'", v.to_type()),
        }
    }

    pub fn to_map(&self) -> crate::Map<'a> {
        match self {
            Self::Map(v) => v.clone(),
            v => panic!("called 'to_map' on '{}'", v.to_type()),
        }
    }
}

impl<'a> AsRef<Value<'a>> for Value<'a> {
    fn as_ref(&self) -> &Value<'a> {
        self
    }
}

impl<'a> crate::TypeOf for Value<'a> {
    fn type_of() -> crate::Type {
        crate::Type::Any
    }
}

impl<'a> crate::ToType for Value<'a> {
    fn to_type(&self) -> crate::Type {
        match self {
            Self::Bool(v) => v.to_type(),
            Self::Number(v) => v.to_type(),
            Self::Str(v) => v.to_type(),
            Self::Map(v) => v.to_type(),
            Self::Mut(v) => v.to_type(),
            Self::Ref(v) => v.to_type(),
            Self::Dynamic(v) => v.to_type(),
            Self::Null => panic!("called 'ToType::to_type' on '<null>'"),
        }
    }
}

impl<'a> crate::ToValue for Value<'a> {
    fn to_value(&self) -> crate::Value<'_> {
        self.clone()
    }
}

impl<'a> PartialEq for Value<'a> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Bool(v) => other.as_bool() == Some(v),
            Self::Number(v) => other.as_number() == Some(v),
            Self::Str(v) => other.as_str() == Some(v),
            Self::Map(v) => other.as_map() == Some(v),
            Self::Mut(v) => other.is_mut() && other.to_mut() == *v,
            Self::Ref(v) => other.is_ref() && other.to_ref() == *v,
            Self::Null => other.is_null(),
            _ => false,
        }
    }
}

impl<'a> PartialEq<dyn crate::ToValue> for Value<'a> {
    fn eq(&self, other: &dyn crate::ToValue) -> bool {
        self.eq(&other.to_value())
    }
}

impl<'a> std::ops::Index<usize> for Value<'a> {
    type Output = Self;

    fn index(&self, _index: usize) -> &Self::Output {
        match self {
            Self::Ref(v) => v.index(_index),
            Self::Mut(v) => v.index(_index),
            _ => panic!("called 'Index<usize>::index' on '{}'", self.to_type()),
        }
    }
}

impl<'a> std::ops::Index<&'a str> for Value<'a> {
    type Output = Self;

    fn index(&self, index: &'a str) -> &Self::Output {
        match self {
            Self::Map(v) => v.index(&crate::Value::Str(crate::Str(index))),
            _ => panic!("called 'Index<&str>::index' on '{}'", self.to_type()),
        }
    }
}

impl<'a> std::ops::Index<&Self> for Value<'a> {
    type Output = Self;

    fn index(&self, index: &Self) -> &Self::Output {
        match self {
            Self::Map(v) => v.index(index),
            _ => panic!("called 'Index<&Value>::index' on '{}'", self.to_type()),
        }
    }
}

impl<'a> std::fmt::Display for Value<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(v) => write!(f, "{}", v),
            Self::Number(v) => write!(f, "{}", v),
            Self::Str(v) => write!(f, "{}", v),
            Self::Map(v) => write!(f, "{}", v),
            Self::Mut(v) => write!(f, "{}", v),
            Self::Ref(v) => write!(f, "{}", v),
            Self::Dynamic(v) => write!(f, "{}", v),
            Self::Null => write!(f, "<null>"),
        }
    }
}

impl<'a> Eq for Value<'a> {}

impl<'a> Ord for Value<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::ops::Deref;

        let ord = match (self, other) {
            (Self::Bool(a), Self::Bool(b)) => a.partial_cmp(b),
            (Self::Number(a), Self::Number(b)) => a.partial_cmp(b),
            (Self::Str(a), Self::Str(b)) => a.0.partial_cmp(b.0),
            (Self::Mut(a), _) => a.deref().partial_cmp(other),
            (Self::Ref(a), _) => a.deref().partial_cmp(other),
            (_, Self::Mut(b)) => self.partial_cmp(b.deref()),
            (_, Self::Ref(b)) => self.partial_cmp(b.deref()),
            _ => None,
        };

        match ord {
            None => panic!("called 'cmp' on '{}'", self.to_type()),
            Some(v) => v,
        }
    }
}

impl<'a> PartialOrd for Value<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(feature = "serde")]
impl<'a> serde::Serialize for Value<'a> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Bool(v) => v.serialize(s),
            Self::Number(v) => v.serialize(s),
            Self::Str(v) => v.0.serialize(s),
            Self::Map(v) => v.serialize(s),
            Self::Mut(v) => v.value.serialize(s),
            Self::Ref(v) => v.value.serialize(s),
            Self::Dynamic(v) => v.serialize(s),
            Self::Null => s.serialize_none(),
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
