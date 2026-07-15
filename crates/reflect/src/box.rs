pub trait ToBox<T> {
    fn to_box(&self) -> Box<T>;
}

impl<T> ToBox<T> for T
where
    T: Clone,
{
    fn to_box(&self) -> Box<T> {
        Box::new(self.clone())
    }
}

impl<T: crate::ToValue> crate::ToValue for std::sync::Arc<T> {
    fn to_value_ref(&self) -> crate::ValueRef<'_> {
        self.as_ref().to_value_ref()
    }
}
