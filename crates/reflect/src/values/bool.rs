impl From<bool> for crate::Value<'static> {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl crate::ToValue for bool {
    fn to_value(&self) -> crate::Value<'static> {
        crate::Value::Bool(*self)
    }
}

impl crate::Value<'_> {
    pub fn is_true(&self) -> bool {
        self.to_bool() == Some(true)
    }

    pub fn is_false(&self) -> bool {
        self.to_bool() == Some(false)
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    pub fn truthy() {
        let value = value_of!(true);
        assert!(value.is_bool());
        assert!(value.is_true());
        assert_eq!(value.to_bool(), Some(true));
    }

    #[test]
    pub fn falsy() {
        let value = value_of!(false);
        assert!(value.is_bool());
        assert!(value.is_false());
        assert_eq!(value.to_bool(), Some(false));
    }
}
