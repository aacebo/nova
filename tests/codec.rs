#![cfg(feature = "codec")]

mod common;

use std::collections::BTreeMap;

use common::Recorder;
use nova::codec::Codec;
#[cfg(feature = "fs")]
use nova::fs::FileSystem;

fn sample() -> nova::Value {
    nova::Value::from_serialize(BTreeMap::from([("name", "nova"), ("kind", "codec")]))
}

fn run(recorder: &Recorder, manifest: nova::Manifest) {
    let runtime = nova::new()
        .observe(recorder.clone())
        .json()
        .yaml()
        .routine(manifest)
        .build()
        .unwrap();

    runtime.call("t", nova::args!()).unwrap();
}

fn routine() -> nova::build::ManifestBuilder {
    nova::manifest().name("t").on([nova::Trigger::Run { priority: None }])
}

#[test]
fn json_round_trips_a_map() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .var("x", sample())
            .step(nova::step().run("{{ info(json.decode(json.encode(x)).name) }}"))
            .build(),
    );
    assert!(recorder.messages().iter().any(|m| m == "nova"), "{:?}", recorder.messages());
}

#[test]
fn json_encode_produces_expected_shape() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .var("x", nova::Value::from_serialize(BTreeMap::from([("a", 1)])))
            .step(nova::step().run("{{ info(json.encode(x)) }}"))
            .build(),
    );
    assert!(
        recorder.messages().iter().any(|m| m == "{\"a\":1}"),
        "{:?}",
        recorder.messages()
    );
}

#[test]
fn json_decode_rejects_malformed() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine().step(nova::step().run("{{ json.decode('{not json') }}")).build(),
    );
    assert!(recorder.has_error());
}

#[test]
fn yaml_round_trips_a_map() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .var("x", sample())
            .step(nova::step().run("{{ info(yaml.decode(yaml.encode(x)).name) }}"))
            .build(),
    );
    assert!(recorder.messages().iter().any(|m| m == "nova"), "{:?}", recorder.messages());
}

#[test]
fn json_and_yaml_interoperate_through_value() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run("{{ info(yaml.decode(yaml.encode(json.decode('{\"n\": 7}'))).n) }}"))
            .build(),
    );
    assert!(recorder.messages().iter().any(|m| m == "7"), "{:?}", recorder.messages());
}

#[test]
#[cfg(feature = "fs")]
fn json_encode_write_read_decode_round_trips_through_fs() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.json");
    let recorder = Recorder::new();

    let runtime = nova::new()
        .observe(recorder.clone())
        .json()
        .yaml()
        .fs()
        .routine(
            routine()
                .var("x", sample())
                .var("path", path.to_str().unwrap())
                .step(nova::step().name("write").run("{{ fs.write(path, json.encode(x)) }}"))
                .step(nova::step().name("read").run("{{ info(json.decode(fs.read(path)).name) }}"))
                .build(),
        )
        .build()
        .unwrap();

    runtime.call("t", nova::args!()).unwrap();
    drop(runtime);

    assert!(recorder.messages().iter().any(|m| m == "nova"), "{:?}", recorder.messages());
    assert!(!recorder.has_error());
}
