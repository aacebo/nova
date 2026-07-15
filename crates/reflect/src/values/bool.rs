impl From<bool> for crate::Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl<'a> From<bool> for crate::ValueRef<'a> {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl crate::ToValue for bool {
    fn to_value_ref(&self) -> crate::ValueRef<'static> {
        crate::ValueRef::Bool(*self)
    }
}

impl crate::ValueRef<'_> {
    pub fn is_true(&self) -> bool {
        self.to_bool() == Some(true)
    }

    pub fn is_false(&self) -> bool {
        self.to_bool() == Some(false)
    }
}

impl crate::Value {
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
