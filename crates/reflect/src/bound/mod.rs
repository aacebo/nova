mod lifetime;
mod r#trait;

pub use lifetime::*;
pub use r#trait::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Bound {
    Trait(TraitBound),
    Lifetime(LifetimeBound),
}

impl Bound {
    pub fn name(&self) -> String {
        match self {
            Self::Trait(v) => v.path.to_string(),
            Self::Lifetime(v) => v.name.clone(),
        }
    }

    pub fn is_trait(&self) -> bool {
        matches!(self, Self::Trait(_))
    }

    pub fn is_lifetime(&self) -> bool {
        matches!(self, Self::Lifetime(_))
    }

    pub fn to_trait(&self) -> Option<TraitBound> {
        match self {
            Self::Trait(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn to_lifetime(&self) -> Option<LifetimeBound> {
        match self {
            Self::Lifetime(v) => Some(v.clone()),
            _ => None,
        }
    }
}

impl std::fmt::Display for Bound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trait(v) => write!(f, "{}", v),
            Self::Lifetime(v) => write!(f, "{}", v),
        }
    }
}
