use crate::TypeOf;

impl<T, E> crate::TypeOf for Result<T, E> {
    fn type_of() -> crate::Type {
        crate::enum_type()
            .path(crate::Path::from("core::result"))
            .name("Result")
            .visibility(crate::Visibility::Public(crate::Public::Full))
            .generics(crate::Generics::from([
                crate::type_param().name("T").build().to_generic(),
                crate::type_param().name("E").build().to_generic(),
            ]))
            .build()
            .to_type()
    }
}

impl<T, E> crate::ToType for Result<T, E> {
    fn to_type(&self) -> crate::Type {
        Result::<T, E>::type_of()
    }
}

impl<T: crate::ToValue, E: crate::ToValue> crate::ToValue for Result<T, E> {
    fn to_value(&self) -> crate::Value<'_> {
        match self {
            Err(err) => err.to_value(),
            Ok(v) => v.to_value(),
        }
    }
}
