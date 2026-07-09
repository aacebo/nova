#![cfg(feature = "fs")]

use nova::{KArgs, Value};
use nova_fs::FileSystem;

fn runtime() -> nova::Runtime {
    nova::new().import(FileSystem).unwrap().build().unwrap()
}

#[test]
fn write_then_read_string_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hello.txt");
    let path = path.to_str().unwrap();
    let rt = runtime();
    let scope = rt.scope();

    scope
        .call("fs.write", [Value::from(path), Value::from("hello world")], KArgs::new())
        .unwrap();

    let out = scope
        .call("fs.read", [Value::from(path)], KArgs::new())
        .unwrap()
        .expect("fs.read should return a value");

    assert_eq!(out.as_str(), Some("hello world"));
}

#[test]
fn write_overwrites_then_read_returns_latest() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    let path = path.to_str().unwrap();
    let rt = runtime();
    let scope = rt.scope();

    scope
        .call("fs.write", [Value::from(path), Value::from("first")], KArgs::new())
        .unwrap();
    scope
        .call("fs.write", [Value::from(path), Value::from("second")], KArgs::new())
        .unwrap();

    let out = scope.call("fs.read", [Value::from(path)], KArgs::new()).unwrap().unwrap();
    assert_eq!(out.as_str(), Some("second"));
}
