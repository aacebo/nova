#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
pub struct TypeId(std::sync::Arc<str>);

impl TypeId {
    pub(crate) fn from_str(value: &str) -> Self {
        Self(std::sync::Arc::from(value))
    }

    pub(crate) fn from_string(value: String) -> Self {
        Self(std::sync::Arc::from(value))
    }
}

impl std::fmt::Display for TypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Eq for TypeId {}

impl PartialEq for TypeId {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0) || self.0 == other.0
    }
}

impl PartialEq<&str> for TypeId {
    fn eq(&self, other: &&str) -> bool {
        &*self.0 == *other
    }
}

impl PartialEq<String> for TypeId {
    fn eq(&self, other: &String) -> bool {
        &*self.0 == other.as_str()
    }
}

impl Ord for TypeId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for TypeId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl AsRef<TypeId> for TypeId {
    fn as_ref(&self) -> &TypeId {
        self
    }
}

impl AsMut<TypeId> for TypeId {
    fn as_mut(&mut self) -> &mut TypeId {
        self
    }
}

impl std::ops::Deref for TypeId {
    type Target = Self;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl std::ops::DerefMut for TypeId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}

impl std::hash::Hash for TypeId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
