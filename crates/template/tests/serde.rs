use nova_template::Pointer;

#[test]
fn deserializes_scalars() {
    let s: Pointer = serde_json::from_str(r#""hi""#).unwrap();
    assert_eq!(s.value().to_string(), "hi");

    let n: Pointer = serde_json::from_str("42").unwrap();
    assert_eq!(n.value().to_u64(), Some(42));

    let b: Pointer = serde_json::from_str("true").unwrap();
    assert_eq!(b.value().to_bool(), Some(true));

    let z: Pointer = serde_json::from_str("null").unwrap();
    assert!(z.value().is_null());
}

#[test]
fn deserializes_a_map() {
    let s: Pointer = serde_json::from_str(r#"{"name":"alex","age":30}"#).unwrap();
    let v = s.value();
    let map = v.as_map().unwrap();
    assert_eq!(map.len(), 2);
}

#[test]
fn deserializes_a_nested_structure() {
    let s: Pointer = serde_json::from_str(r#"{"a":{"b":[1,2,3]}}"#).unwrap();
    assert!(s.value().is_map());
}

#[test]
fn round_trips_through_serde() {
    let s: Pointer = serde_json::from_str(r#"{"k":"v"}"#).unwrap();
    let out = serde_json::to_string(&s).unwrap();
    assert_eq!(out, r#"{"k":"v"}"#);
}
