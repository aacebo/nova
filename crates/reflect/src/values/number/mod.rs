mod float;
mod int;

pub use float::*;
pub use int::*;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub enum Number {
    Int(Int),
    Float(Float),
}

#[cfg(feature = "serde")]
impl serde::Serialize for Number {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Int(v) => v.serialize(s),
            Self::Float(v) => v.serialize(s),
        }
    }
}

impl Number {
    pub fn to_type(&self) -> crate::Type {
        match self {
            Self::Int(v) => v.to_type(),
            Self::Float(v) => v.to_type(),
        }
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(_))
    }
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    pub fn to_int(&self) -> Option<Int> {
        self.as_int().copied()
    }

    pub fn as_int(&self) -> Option<&Int> {
        match self {
            Self::Int(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_float(&self) -> Option<Float> {
        self.as_float().copied()
    }

    pub fn as_float(&self) -> Option<&Float> {
        match self {
            Self::Float(v) => Some(v),
            _ => None,
        }
    }
}

impl crate::ToValue for Number {
    fn to_value(&self) -> crate::Value<'static> {
        crate::Value::Number(*self)
    }
}

impl PartialEq<crate::Value<'_>> for Number {
    fn eq(&self, other: &crate::Value<'_>) -> bool {
        other.as_number() == Some(self)
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{}", v),
            Self::Float(v) => write!(f, "{}", v),
        }
    }
}
