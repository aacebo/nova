#![cfg(feature = "fs")]

use nova::KArgs;
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

    rt.call("fs.write", [path, "hello world"], KArgs::new()).unwrap();

    let out = rt
        .func("fs.read", [path], KArgs::new())
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

    rt.call("fs.write", [path, "first"], KArgs::new()).unwrap();
    rt.call("fs.write", [path, "second"], KArgs::new()).unwrap();

    let out = rt.func("fs.read", [path], KArgs::new()).unwrap().unwrap();
    assert_eq!(out.as_str(), Some("second"));
}
