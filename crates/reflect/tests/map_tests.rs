use std::collections::HashMap;

use nova_reflect::{TypeOf, type_of};
use nova_reflect_macros::*;

#[derive(Debug, Clone, Reflect)]
pub struct Tester(HashMap<u8, i8>);

#[test]
pub fn should_reflect_map() {
    let ty = type_of!(Tester).to_struct().unwrap();
    let inner = &ty.fields()[0];
    assert_eq!(inner.ty(), &type_of!(HashMap<u8, i8>));
}
