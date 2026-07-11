#![cfg(feature = "json")]

use nova_reflect::{Value, value_of};
use nova_reflect_macros::*;

#[derive(Debug, Clone, Reflect)]
pub struct User {
    pub name: String,
    pub age: u32,
    pub admin: bool,
}

#[test]
pub fn reflected_struct_serializes_to_json() {
    let user = User {
        name: String::from("alex"),
        age: 30,
        admin: true,
    };

    let owned = user.clone();
    let value = value_of!(owned);
    let json: serde_json::Value = (&value).into();

    assert_eq!(
        json,
        serde_json::json!({
            "name": "alex",
            "age": 30,
            "admin": true,
        })
    );
}

#[test]
pub fn json_object_into_value_round_trips() {
    let json = serde_json::json!({
        "name": "alex",
        "nested": { "count": 3 },
        "flag": true,
    });

    let value = Value::from(json.clone());
    assert!(value.is_map());

    let back: serde_json::Value = value.into();
    assert_eq!(back, json);
}

#[test]
pub fn json_array_within_object_becomes_index_map() {
    let value = Value::from(serde_json::json!({ "tags": ["a", "b"] }));
    let back: serde_json::Value = value.into();

    assert_eq!(back, serde_json::json!({ "tags": { "0": "a", "1": "b" } }));
}

#[test]
pub fn json_scalars_into_value() {
    assert!(Value::from(serde_json::json!(null)).is_null());
    assert!(Value::from(serde_json::json!(true)).is_bool());
    assert!(Value::from(serde_json::json!(42)).is_number());
    assert!(Value::from(serde_json::json!(-1)).is_number());
    assert!(Value::from(serde_json::json!(3.5)).is_number());
    assert!(Value::from(serde_json::json!("hi")).is_str());
}

#[test]
pub fn json_number_variants_preserved() {
    for json in [
        serde_json::json!(-7),
        serde_json::json!(7),
        serde_json::json!(9_000_000_000_000_000_000_u64),
        serde_json::json!(2.5),
    ] {
        let value = Value::from(json.clone());
        let back: serde_json::Value = value.into();
        assert_eq!(back, json);
    }
}

#[test]
pub fn json_array_indexable_via_index_keys() {
    let value = Value::from(serde_json::json!(["a", "b", "c"]));

    assert!(value.is_map());
    assert_eq!(value.len(), 3);

    let back: serde_json::Value = value.into();
    assert_eq!(back, serde_json::json!({ "0": "a", "1": "b", "2": "c" }));
}
