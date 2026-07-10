#![cfg(feature = "fs")]

use nova::fs::FileSystem;
use nova::{KArgs, Object, Value};

fn runtime() -> nova::Runtime {
    nova::new().import(FileSystem).unwrap().build().unwrap()
}

#[test]
fn write_then_read_string_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hello.txt");
    let path = path.to_str().unwrap();
    let rt = runtime();
    let scope = rt.scope().fork("t", vec![], KArgs::new());

    scope.set_local("path", Object::value(Value::from(path)));
    scope.set_local("data", Object::value(Value::from("hello world")));
    scope.render_str("{{ fs.write(path, data) }}").unwrap();

    let out = scope.render_str("{{ fs.read(path) }}").unwrap();
    assert_eq!(out, "hello world");
}

#[test]
fn write_overwrites_then_read_returns_latest() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    let path = path.to_str().unwrap();
    let rt = runtime();
    let scope = rt.scope().fork("t", vec![], KArgs::new());

    scope.set_local("path", Object::value(Value::from(path)));
    scope.set_local("data", Object::value(Value::from("first")));
    scope.render_str("{{ fs.write(path, data) }}").unwrap();
    scope.set_local("data", Object::value(Value::from("second")));
    scope.render_str("{{ fs.write(path, data) }}").unwrap();

    let out = scope.render_str("{{ fs.read(path) }}").unwrap();
    assert_eq!(out, "second");
}
