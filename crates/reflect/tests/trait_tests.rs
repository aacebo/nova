use nova_reflect::{TypeOf, type_of, value_of};
use nova_reflect_macros::*;

#[allow(dead_code)]
#[reflect(a = "b")]
trait Hello<T = String> {
    fn world(&self, a: u8) -> bool;
}

#[test]
pub fn should_reflect_trait() {
    let ty = type_of!(dyn Hello);

    assert!(ty.is_trait());
    assert_eq!(ty.len(), 1);

    let tr = ty.to_trait().unwrap();

    assert!(tr.has("world"));
    assert_eq!(tr.get("world").unwrap().params().len(), 2);
    assert!(tr.get("world").unwrap().has_param("self"));
    assert!(tr.get("world").unwrap().has_param("a"));
    assert!(tr.meta().has("a"));
    assert_eq!(tr.meta().get("a").unwrap(), &value_of!("b"));
    assert_eq!(tr.generics().len(), 1);
    assert_eq!(tr.generics()[0].to_type().unwrap().name(), "T");
    assert_eq!(tr.generics()[0].to_type().unwrap().default().unwrap(), &type_of!(String));
}

#[allow(dead_code)]
#[reflect]
trait Borrowing {
    fn shared(&self, x: &i32) -> bool;
    fn exclusive(&mut self, y: &mut i32);
}

#[test]
pub fn should_reflect_reference_params() {
    let ty = type_of!(dyn Borrowing).to_trait().unwrap();

    let shared_x = ty.get("shared").unwrap().param("x");
    assert!(shared_x.ty().is_ref());
    assert!(shared_x.ty().is_ref_of(type_of!(i32)));

    let exclusive_y = ty.get("exclusive").unwrap().param("y");
    assert!(exclusive_y.ty().is_ref());
    assert!(exclusive_y.ty().is_ref_mut());
}
