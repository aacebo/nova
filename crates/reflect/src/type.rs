#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Type {
    Any,
    Bool(crate::BoolType),
    Enum(std::sync::Arc<crate::EnumType>),
    Number(crate::NumberType),
    Str(crate::StrType),
    Ref(crate::RefType),
    Slice(crate::SliceType),
    Map(std::sync::Arc<crate::MapType>),
    Struct(std::sync::Arc<crate::StructType>),
    This(crate::ThisType),
    Tuple(std::sync::Arc<crate::TupleType>),
    Trait(std::sync::Arc<crate::TraitType>),
    Mut(crate::MutType),
    Mod(std::sync::Arc<crate::ModType>),
    Void,
}

impl Type {
    pub fn id(&self) -> crate::TypeId {
        match self {
            Self::Any => crate::TypeId::from_str("any"),
            Self::Bool(v) => v.id(),
            Self::Enum(v) => v.id(),
            Self::Number(v) => v.id(),
            Self::Str(v) => v.id(),
            Self::Ref(v) => v.id(),
            Self::Slice(v) => v.id(),
            Self::Map(v) => v.id(),
            Self::Struct(v) => v.id(),
            Self::This(v) => v.id(),
            Self::Tuple(v) => v.id(),
            Self::Trait(v) => v.id(),
            Self::Mut(v) => v.id(),
            Self::Mod(v) => v.id(),
            Self::Void => crate::TypeId::from_str("void"),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Enum(v) => v.len(),
            Self::Slice(v) => v.len(),
            Self::Struct(v) => v.len(),
            Self::Tuple(v) => v.len(),
            Self::Trait(v) => v.len(),
            Self::Mod(v) => v.len(),
            _ => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn path(&self) -> Option<&crate::Path> {
        match self {
            Self::Map(v) => v.path(),
            Self::Struct(v) => Some(v.path()),
            Self::Enum(v) => Some(v.path()),
            Self::Trait(v) => Some(v.path()),
            Self::Mod(v) => Some(v.path()),
            _ => None,
        }
    }

    pub fn meta(&self) -> Option<&crate::MetaData> {
        match self {
            Self::Map(v) => v.meta(),
            Self::Struct(v) => Some(v.meta()),
            Self::Enum(v) => Some(v.meta()),
            Self::Trait(v) => Some(v.meta()),
            Self::Mod(v) => Some(v.meta()),
            _ => None,
        }
    }

    pub fn to_item(&self) -> crate::Item {
        crate::Item::Type(self.clone())
    }

    pub fn is_any(&self) -> bool {
        matches!(self, Self::Any)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, Self::Enum(_))
    }

    pub fn is_ref(&self) -> bool {
        matches!(self, Self::Ref(_))
    }

    pub fn is_ref_of(&self, ty: Self) -> bool {
        match self {
            Self::Ref(v) => v.is_ref_of(ty),
            _ => false,
        }
    }

    pub fn is_ref_self(&self) -> bool {
        match self {
            Self::Ref(v) => v.ty().is_self(),
            _ => false,
        }
    }

    pub fn is_ref_mut(&self) -> bool {
        match self {
            Self::Ref(v) => v.ty().is_mut(),
            _ => false,
        }
    }

    pub fn is_ref_mut_self(&self) -> bool {
        match self {
            Self::Ref(v) => v.ty().is_mut_self(),
            _ => false,
        }
    }

    pub fn is_slice(&self) -> bool {
        matches!(self, Self::Slice(_))
    }

    pub fn is_slice_of(&self, ty: Self) -> bool {
        match self {
            Self::Slice(v) => v.is_slice_of(ty),
            _ => false,
        }
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Self::Struct(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn is_int(&self) -> bool {
        match self {
            Self::Number(v) => v.is_int(),
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Self::Number(v) => v.is_float(),
            _ => false,
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            Self::Number(v) => v.is_signed(),
            _ => false,
        }
    }

    pub fn is_str(&self) -> bool {
        matches!(self, Self::Str(_))
    }

    pub fn is_self(&self) -> bool {
        matches!(self, Self::This(_))
    }

    pub fn is_tuple(&self) -> bool {
        matches!(self, Self::Tuple(_))
    }

    pub fn is_trait(&self) -> bool {
        matches!(self, Self::Trait(_))
    }

    pub fn is_mut(&self) -> bool {
        matches!(self, Self::Mut(_))
    }

    pub fn is_mut_of(&self, ty: crate::Type) -> bool {
        match self {
            Self::Mut(v) => v.is_mut_of(ty),
            _ => false,
        }
    }

    pub fn is_mut_self(&self) -> bool {
        match self {
            Self::Mut(v) => v.ty().is_self(),
            _ => false,
        }
    }

    pub fn is_mod(&self) -> bool {
        matches!(self, Self::Mod(_))
    }

    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }

    pub fn to_bool(&self) -> Option<crate::BoolType> {
        self.as_bool().copied()
    }

    pub fn as_bool(&self) -> Option<&crate::BoolType> {
        match self {
            Self::Bool(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_enum(&self) -> Option<crate::EnumType> {
        self.as_enum().cloned()
    }

    pub fn as_enum(&self) -> Option<&crate::EnumType> {
        match self {
            Self::Enum(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    pub fn to_ref(&self) -> Option<crate::RefType> {
        self.as_ref().cloned()
    }

    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> Option<&crate::RefType> {
        match self {
            Self::Ref(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_slice(&self) -> Option<crate::SliceType> {
        self.as_slice().cloned()
    }

    pub fn as_slice(&self) -> Option<&crate::SliceType> {
        match self {
            Self::Slice(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_struct(&self) -> Option<crate::StructType> {
        self.as_struct().cloned()
    }

    pub fn as_struct(&self) -> Option<&crate::StructType> {
        match self {
            Self::Struct(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    pub fn to_number(&self) -> Option<crate::NumberType> {
        self.as_number().copied()
    }

    pub fn as_number(&self) -> Option<&crate::NumberType> {
        match self {
            Self::Number(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_int(&self) -> Option<crate::IntType> {
        self.as_int().copied()
    }

    pub fn as_int(&self) -> Option<&crate::IntType> {
        match self {
            Self::Number(v) => v.as_int(),
            _ => None,
        }
    }

    pub fn to_float(&self) -> Option<crate::FloatType> {
        self.as_float().copied()
    }

    pub fn as_float(&self) -> Option<&crate::FloatType> {
        match self {
            Self::Number(v) => v.as_float(),
            _ => None,
        }
    }

    pub fn to_str(&self) -> Option<crate::StrType> {
        self.as_str().copied()
    }

    pub fn as_str(&self) -> Option<&crate::StrType> {
        match self {
            Self::Str(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_self(&self) -> Option<crate::ThisType> {
        self.as_self().cloned()
    }

    pub fn as_self(&self) -> Option<&crate::ThisType> {
        match self {
            Self::This(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_tuple(&self) -> Option<crate::TupleType> {
        self.as_tuple().cloned()
    }

    pub fn as_tuple(&self) -> Option<&crate::TupleType> {
        match self {
            Self::Tuple(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    pub fn to_trait(&self) -> Option<crate::TraitType> {
        self.as_trait().cloned()
    }

    pub fn as_trait(&self) -> Option<&crate::TraitType> {
        match self {
            Self::Trait(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    pub fn to_mut(&self) -> Option<crate::MutType> {
        self.as_mut().cloned()
    }

    pub fn as_mut(&self) -> Option<&crate::MutType> {
        match self {
            Self::Mut(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_mod(&self) -> Option<crate::ModType> {
        self.as_mod().cloned()
    }

    pub fn as_mod(&self) -> Option<&crate::ModType> {
        match self {
            Self::Mod(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    pub fn to_map(&self) -> Option<crate::MapType> {
        self.as_map().cloned()
    }

    pub fn as_map(&self) -> Option<&crate::MapType> {
        match self {
            Self::Map(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    pub fn assignable_to(&self, ty: Self) -> bool {
        match self {
            Self::Bool(v) => v.assignable_to(ty),
            Self::Enum(v) => v.assignable_to(ty),
            Self::Number(v) => v.assignable_to(ty),
            Self::Str(v) => v.assignable_to(ty),
            Self::Ref(v) => v.assignable_to(ty),
            Self::Slice(v) => v.assignable_to(ty),
            Self::Struct(v) => v.assignable_to(ty),
            Self::This(v) => v.assignable_to(ty),
            Self::Tuple(v) => v.assignable_to(ty),
            Self::Trait(v) => v.assignable_to(ty),
            Self::Mut(v) => v.assignable_to(ty),
            Self::Mod(v) => v.assignable_to(ty),
            Self::Map(v) => v.assignable_to(ty),
            _ => false,
        }
    }

    pub fn convertable_to(&self, ty: Self) -> bool {
        match self {
            Self::Bool(v) => v.convertable_to(ty),
            Self::Enum(v) => v.convertable_to(ty),
            Self::Number(v) => v.convertable_to(ty),
            Self::Str(v) => v.convertable_to(ty),
            Self::Ref(v) => v.convertable_to(ty),
            Self::Slice(v) => v.convertable_to(ty),
            Self::Struct(v) => v.convertable_to(ty),
            Self::This(v) => v.convertable_to(ty),
            Self::Tuple(v) => v.convertable_to(ty),
            Self::Trait(v) => v.convertable_to(ty),
            Self::Mut(v) => v.convertable_to(ty),
            Self::Mod(v) => v.convertable_to(ty),
            Self::Map(v) => v.convertable_to(ty),
            _ => false,
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any => write!(f, "any"),
            Self::Bool(v) => write!(f, "{}", v),
            Self::Enum(v) => write!(f, "{}", v),
            Self::Number(v) => write!(f, "{}", v),
            Self::Str(v) => write!(f, "{}", v),
            Self::Ref(v) => write!(f, "{}", v),
            Self::Slice(v) => write!(f, "{}", v),
            Self::Struct(v) => write!(f, "{}", v),
            Self::This(v) => write!(f, "{}", v),
            Self::Tuple(v) => write!(f, "{}", v),
            Self::Trait(v) => write!(f, "{}", v),
            Self::Mut(v) => write!(f, "{}", v),
            Self::Mod(v) => write!(f, "{}", v),
            Self::Map(v) => write!(f, "{}", v),
            Self::Void => write!(f, "void"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    pub fn as_returns_none_on_mismatch() {
        assert_eq!(Type::Any.as_map(), None);
        assert_eq!(Type::Any.as_number(), None);
        assert_eq!(Type::Void.as_bool(), None);
        assert_eq!(type_of!(i32).as_str(), None);
    }

    #[test]
    pub fn as_returns_some_on_match() {
        assert!(type_of!(bool).as_bool().is_some());
        assert!(type_of!(i32).as_number().is_some());
        assert!(type_of!(i32).as_int().is_some());
        assert!(type_of!(f64).as_float().is_some());
    }
}
