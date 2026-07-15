impl<'a> crate::ToType for &'a [crate::ValueRef<'a>] {
    fn to_type(&self) -> crate::Type {
        let elem = self.first().map(crate::ToType::to_type).unwrap_or(crate::Type::Any);

        crate::Type::Slice(crate::SliceType {
            ty: std::sync::Arc::new(elem),
            capacity: None,
        })
    }
}

impl<'a> crate::Sequence for &'a [crate::ValueRef<'a>] {
    fn len(&self) -> usize {
        <[crate::ValueRef<'a>]>::len(self)
    }

    fn index(&self, i: usize) -> crate::ValueRef<'_> {
        match self.get(i) {
            None => crate::ValueRef::Null,
            Some(v) => v.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    pub fn value_slice_to_value() {
        let values: [ValueRef<'_>; 3] = [value_of!(&1_i32), value_of!(&2_i32), value_of!(&3_i32)];
        let slice: &[ValueRef<'_>] = &values;

        assert_eq!(<&[ValueRef<'_>] as Sequence>::len(&slice), 3);

        for i in 0..Sequence::len(&slice) {
            let v = Sequence::index(&slice, i);
            assert!(v.is_i32());
            assert_eq!(i + 1, v.to_i32().unwrap() as usize);
        }
    }
}
