#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Item {
    Type(crate::Type),
    Impl(crate::Impl),
}

impl Item {
    pub fn id(&self) -> crate::TypeId {
        match self {
            Self::Type(v) => v.id(),
            Self::Impl(v) => v.id(),
        }
    }

    pub fn is_type(&self) -> bool {
        matches!(self, Self::Type(_))
    }

    pub fn is_impl(&self) -> bool {
        matches!(self, Self::Impl(_))
    }

    pub fn to_type(&self) -> Option<crate::Type> {
        match self {
            Self::Type(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn to_impl(&self) -> Option<crate::Impl> {
        match self {
            Self::Impl(v) => Some(v.clone()),
            _ => None,
        }
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type(v) => write!(f, "{}", v),
            Self::Impl(v) => write!(f, "{}", v),
        }
    }
}
