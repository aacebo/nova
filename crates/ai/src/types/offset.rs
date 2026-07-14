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
    fn type_of() -> nova_core::Type {
        nova_core::Type::Any
    }
}

impl nova_core::ToType for Offset {
    fn to_type(&self) -> nova_core::Type {
        nova_core::Type::Any
    }
}

impl nova_core::Reflect for Offset {
    fn field(&self, name: &str) -> nova_core::Value<'_> {
        match name {
            "begin" => nova_core::Value::from(self.begin),
            "end" => nova_core::Value::from(self.end),
            _ => nova_core::Value::Undefined,
        }
    }
}

impl nova_core::ToValue for Offset {
    fn to_value(&self) -> nova_core::Value<'_> {
        nova_core::Value::Dynamic(nova_core::Dynamic::from_object(self))
    }
}
