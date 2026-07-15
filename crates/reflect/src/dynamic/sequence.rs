/// ## Sequence
///
/// implemented by types that can reflect their value/type
/// and the values of their individual elements
pub trait Sequence: std::fmt::Debug + Send + Sync + crate::ToType {
    fn len(&self) -> usize;
    fn index_by_ref(&self, i: usize) -> crate::ValueRef<'_>;

    fn index(&self, i: usize) -> crate::Value {
        self.index_by_ref(i).to_owned()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(&self) -> Box<dyn Iterator<Item = crate::Value> + '_> {
        Box::new((0..self.len()).map(move |i| self.index(i)))
    }
}

impl std::fmt::Display for dyn Sequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;

        for value in self.iter() {
            write!(f, "\n\t{}", value)?;
        }

        write!(f, "\n]")
    }
}

impl<T> crate::TypeOf for Vec<T>
where
    T: crate::TypeOf,
{
    fn type_of() -> crate::Type {
        crate::Type::Slice(crate::SliceType {
            ty: std::sync::Arc::new(T::type_of()),
            capacity: None,
        })
    }
}

impl<T> crate::ToType for Vec<T>
where
    T: crate::TypeOf,
{
    fn to_type(&self) -> crate::Type {
        <Vec<T> as crate::TypeOf>::type_of()
    }
}

impl<T> crate::ToValue for Vec<T>
where
    T: std::fmt::Debug + Send + Sync + crate::TypeOf + crate::ToValue + 'static,
{
    fn to_value_ref(&self) -> crate::ValueRef<'_> {
        crate::ValueRef::Dynamic(crate::DynamicRef::from_sequence(self))
    }
}

impl<T> crate::Sequence for Vec<T>
where
    T: std::fmt::Debug + Send + Sync + crate::TypeOf + crate::ToValue + 'static,
{
    fn len(&self) -> usize {
        self.len()
    }

    fn index_by_ref(&self, i: usize) -> crate::ValueRef<'_> {
        match self.get(i) {
            None => crate::ValueRef::Null,
            Some(v) => v.to_value_ref(),
        }
    }

    fn index(&self, i: usize) -> crate::Value {
        match self.get(i) {
            None => crate::Value::Null,
            Some(v) => v.to_value(),
        }
    }
}

impl<const N: usize, T> crate::Sequence for [T; N]
where
    T: std::fmt::Debug + Send + Sync + crate::TypeOf + crate::ToValue + 'static,
{
    fn len(&self) -> usize {
        N
    }

    fn index_by_ref(&self, i: usize) -> crate::ValueRef<'_> {
        match self.get(i) {
            None => crate::ValueRef::Null,
            Some(v) => v.to_value_ref(),
        }
    }

    fn index(&self, i: usize) -> crate::Value {
        match self.get(i) {
            None => crate::Value::Null,
            Some(v) => v.to_value(),
        }
    }
}

impl<const N: usize, T> crate::ToValue for [T; N]
where
    T: std::fmt::Debug + Send + Sync + crate::TypeOf + crate::ToValue + 'static,
{
    fn to_value_ref(&self) -> crate::ValueRef<'_> {
        crate::ValueRef::Dynamic(crate::DynamicRef::from_sequence(self))
    }
}

#[cfg(test)]
mod test {
    use crate::{DynamicRef, ToValue};

    #[test]
    pub fn vec_sequence_index_returns_element() {
        let vec = vec![10_i32, 20, 30];
        let dynamic = DynamicRef::from_sequence(&vec);

        assert_eq!(dynamic.len(), 3);
        assert_eq!(dynamic.as_sequence().unwrap().index(1).to_i32(), Some(20));
    }

    #[test]
    pub fn vec_to_value_routes_through_dynamic_sequence() {
        let vec = vec![10_i32, 20, 30];
        let value = vec.to_value();

        assert!(value.is_dynamic());
        let seq = value.as_dynamic().unwrap().as_sequence().unwrap();
        assert_eq!(seq.len(), 3);
        assert_eq!(seq.index(2).to_i32(), Some(30));
    }

    #[test]
    pub fn array_to_value_routes_through_dynamic_sequence() {
        let arr: [i32; 3] = [1, 2, 3];
        let value = arr.to_value();

        assert!(value.is_dynamic());
        let seq = value.as_dynamic().unwrap().as_sequence().unwrap();
        assert_eq!(seq.len(), 3);
        assert_eq!(seq.index(0).to_i32(), Some(1));
    }
}
