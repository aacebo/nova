#[macro_export]
macro_rules! value_of {
    (&$value:expr) => {
        $crate::ToValue::to_value_ref(&$value)
    };
    ($value:expr) => {
        $crate::ToValue::to_value(&$value)
    };
}

pub trait ToValue {
    fn to_value_ref(&self) -> crate::ValueRef<'_>;

    fn to_value(&self) -> crate::Value {
        self.to_value_ref().to_owned()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for dyn ToValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_value_ref().serialize(serializer)
    }
}

#[cfg(test)]
mod test {
    #[test]
    pub fn from_expr() {
        let value = value_of!(1_i8);
        assert!(value.is_i8());
        assert_eq!(value.to_i8(), Some(1));
    }
}
