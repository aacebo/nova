#![cfg(feature = "fs")]

use nova::args;
use nova::fs::FileSystem;

fn runtime() -> nova::Runtime {
    nova::new().import(FileSystem).unwrap().build().unwrap()
}

#[test]
fn write_then_read_string_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hello.txt");
    let path = path.to_str().unwrap();
    let rt = runtime();

    rt.call("fs.write", args!(path, "hello world")).unwrap();

    let out = rt.func("fs.read", args!(path)).unwrap();
    assert_eq!(out.as_str(), Some("hello world"));
}

#[test]
fn write_overwrites_then_read_returns_latest() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    let path = path.to_str().unwrap();
    let rt = runtime();

    rt.call("fs.write", args!(path, "first")).unwrap();
    rt.call("fs.write", args!(path, "second")).unwrap();

    let out = rt.func("fs.read", args!(path)).unwrap();
    assert_eq!(out.as_str(), Some("second"));
}
