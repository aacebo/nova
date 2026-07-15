#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Offset {
    pub begin: u32,
    pub end: u32,
}

impl Offset {
    pub fn new(begin: u32, end: u32) -> Self {
        Self { begin, end }
    }
}

impl std::fmt::Display for Offset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.begin, self.end)
    }
}

impl nova_reflect::TypeOf for Offset {
    fn type_of() -> nova_reflect::Type {
        nova_reflect::Type::Any
    }
}

impl nova_reflect::ToType for Offset {
    fn to_type(&self) -> nova_reflect::Type {
        nova_reflect::Type::Any
    }
}

impl nova_reflect::Object for Offset {
    fn field(&self, name: &str) -> nova_reflect::ValueRef<'_> {
        match name {
            "begin" => nova_reflect::ValueRef::from(self.begin),
            "end" => nova_reflect::ValueRef::from(self.end),
            _ => nova_reflect::ValueRef::Undefined,
        }
    }
}

impl nova_reflect::ToValue for Offset {
    fn to_value_ref(&self) -> nova_reflect::ValueRef<'_> {
        nova_reflect::ValueRef::Dynamic(nova_reflect::DynamicRef::from_object(self))
    }
}
