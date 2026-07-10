#![cfg(feature = "codec")]

use std::collections::BTreeMap;

use nova::codec::Codec;
use nova::{KArgs, Object, Value};

fn runtime() -> nova::Runtime {
    nova::new().import(Codec).unwrap().build().unwrap()
}

fn sample() -> Value {
    Value::from_serialize(BTreeMap::from([("name", "nova"), ("kind", "codec")]))
}

fn scope_with(rt: &nova::Runtime, bindings: &[(&str, Value)]) -> nova::Scope {
    let scope = rt.scope().fork("t", vec![], KArgs::new());
    for (key, value) in bindings {
        scope.set_local(*key, Object::value(value.clone()));
    }
    scope
}

#[test]
fn json_round_trips_a_map() {
    let rt = runtime();
    let scope = scope_with(&rt, &[("x", sample())]);
    let out = scope
        .render_str("{{ json.decode(json.encode(x)).name }}|{{ json.decode(json.encode(x)).kind }}")
        .unwrap();
    assert_eq!(out, "nova|codec");
}

#[test]
fn json_encode_produces_expected_shape() {
    let rt = runtime();
    let scope = scope_with(&rt, &[("x", Value::from_serialize(BTreeMap::from([("a", 1)])))]);
    let out = scope.render_str("{{ json.encode(x) }}").unwrap();
    assert_eq!(out, "{\"a\":1}");
}

#[test]
fn json_decode_rejects_malformed() {
    let rt = runtime();
    let scope = scope_with(&rt, &[]);
    assert!(scope.render_str("{{ json.decode('{not json') }}").is_err());
}

#[test]
fn yaml_round_trips_a_map() {
    let rt = runtime();
    let scope = scope_with(&rt, &[("x", sample())]);
    let out = scope
        .render_str("{{ yaml.decode(yaml.encode(x)).name }}|{{ yaml.decode(yaml.encode(x)).kind }}")
        .unwrap();
    assert_eq!(out, "nova|codec");
}

#[test]
fn json_and_yaml_interoperate_through_value() {
    let rt = runtime();
    let scope = scope_with(&rt, &[]);
    let out = scope
        .render_str("{{ yaml.decode(yaml.encode(json.decode('{\"n\": 7}'))).n }}")
        .unwrap();
    assert_eq!(out, "7");
}

#[test]
#[cfg(feature = "fs")]
fn json_encode_write_read_decode_round_trips_through_fs() {
    let rt = nova::new()
        .import(Codec)
        .unwrap()
        .import(nova::fs::FileSystem)
        .unwrap()
        .build()
        .unwrap();

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.json");
    let path = path.to_str().unwrap();

    let scope = rt.scope().fork("t", vec![], KArgs::new());
    scope.set_local("path", Object::value(Value::from(path)));
    scope.set_local("x", Object::value(sample()));

    scope.render_str("{{ fs.write(path, json.encode(x)) }}").unwrap();
    let out = scope
        .render_str("{{ json.decode(fs.read(path)).name }}|{{ json.decode(fs.read(path)).kind }}")
        .unwrap();
    assert_eq!(out, "nova|codec");
}
