#![cfg(feature = "fs")]

mod common;

use common::Recorder;
use nova::FileSystem;

fn run(recorder: &Recorder, manifest: nova::Manifest) {
    let runtime = nova::new().observe(recorder.clone()).fs().routine(manifest).build().unwrap();

    runtime.call("t", nova::args!()).unwrap();
}

#[test]
fn write_then_read_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hello.txt");
    let recorder = Recorder::new();

    run(
        &recorder,
        nova::manifest()
            .name("t")
            .on([nova::Trigger::Run { priority: None }])
            .var("path", path.to_str().unwrap())
            .step(nova::step().name("write").run("{{ fs.write(path, 'hello world') }}"))
            .step(nova::step().name("read").run("{{ info(fs.read(path)) }}"))
            .build(),
    );

    assert!(
        recorder.messages().iter().any(|m| m == "hello world"),
        "{:?}",
        recorder.messages()
    );
    assert!(!recorder.has_error());
}

#[test]
fn write_overwrites_then_read_returns_latest() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    let recorder = Recorder::new();

    run(
        &recorder,
        nova::manifest()
            .name("t")
            .on([nova::Trigger::Run { priority: None }])
            .var("path", path.to_str().unwrap())
            .step(nova::step().name("first").run("{{ fs.write(path, 'first') }}"))
            .step(nova::step().name("second").run("{{ fs.write(path, 'second') }}"))
            .step(nova::step().name("read").run("{{ info(fs.read(path)) }}"))
            .build(),
    );

    let messages = recorder.messages();
    assert!(messages.iter().any(|m| m == "second"), "{messages:?}");
    assert!(!messages.iter().any(|m| m == "first"), "{messages:?}");
    assert!(!recorder.has_error());
}
