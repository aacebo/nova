mod common;

use common::Recorder;
use nova::Value;
use nova::schema::{Schema, number, object, string};

fn build(manifest: nova::Manifest, recorder: &Recorder) -> Result<nova::Runtime, Box<dyn std::error::Error>> {
    nova::new().observe(recorder.clone()).routine(manifest).build()
}

fn routine() -> nova::build::ManifestBuilder {
    nova::manifest().name("t").on([nova::Trigger::Run { priority: None }])
}

#[test]
fn valid_kwargs_pass() {
    let recorder = Recorder::new();
    let runtime = build(
        routine()
            .args(object().field("name", string()))
            .step(nova::step().run("{{ info('ran') }}"))
            .build(),
        &recorder,
    )
    .unwrap();

    runtime.call("t", nova::args!(name = "x")).unwrap();
    drop(runtime);

    assert!(recorder.messages().iter().any(|m| m == "ran"), "{:?}", recorder.messages());
    assert!(!recorder.has_error());
}

#[test]
fn missing_required_kwarg_fails_and_skips_steps() {
    let recorder = Recorder::new();
    let runtime = build(
        routine()
            .args(object().field("name", string()))
            .step(nova::step().run("{{ info('ran') }}"))
            .build(),
        &recorder,
    )
    .unwrap();

    let result = runtime.call("t", nova::args!());
    drop(runtime);

    assert!(result.is_err());
    assert!(!recorder.messages().iter().any(|m| m == "ran"), "{:?}", recorder.messages());
}

#[test]
fn positional_by_index_passes_and_fails() {
    let pass_recorder = Recorder::new();
    let pass = build(
        routine()
            .args(object().field("0", number()))
            .step(nova::step().run("{{ info('ran') }}"))
            .build(),
        &pass_recorder,
    )
    .unwrap();
    pass.call("t", nova::args!(42)).unwrap();
    drop(pass);
    assert!(pass_recorder.messages().iter().any(|m| m == "ran"));
    assert!(!pass_recorder.has_error());

    let fail_recorder = Recorder::new();
    let fail = build(
        routine()
            .args(object().field("0", number()))
            .step(nova::step().run("{{ info('ran') }}"))
            .build(),
        &fail_recorder,
    )
    .unwrap();
    assert!(fail.call("t", nova::args!("nope")).is_err());
    drop(fail);
    assert!(!fail_recorder.messages().iter().any(|m| m == "ran"));
}

#[test]
fn type_mismatch_fails() {
    let recorder = Recorder::new();
    let runtime = build(
        routine()
            .args(object().field("count", number()))
            .step(nova::step().run("{{ info('ran') }}"))
            .build(),
        &recorder,
    )
    .unwrap();

    assert!(runtime.call("t", nova::args!(count = "not-a-number")).is_err());
    drop(runtime);
}

#[test]
fn no_schema_accepts_anything() {
    let recorder = Recorder::new();
    let runtime = build(routine().step(nova::step().run("{{ info('ran') }}")).build(), &recorder).unwrap();

    runtime.call("t", nova::args!(1, 2, 3, whatever = 123)).unwrap();
    drop(runtime);

    assert!(recorder.messages().iter().any(|m| m == "ran"));
    assert!(!recorder.has_error());
}

#[test]
fn call_step_is_validated_by_target() {
    let recorder = Recorder::new();
    let target = nova::manifest()
        .name("a")
        .on([nova::Trigger::Call])
        .args(object().field("name", string()))
        .step(nova::step().run("{{ info('a ran') }}"))
        .build();

    let caller = routine()
        .step(nova::step().call("a", Vec::<Value>::new(), Vec::<(String, Value)>::new()))
        .build();

    let runtime = nova::new()
        .observe(recorder.clone())
        .routine(target)
        .routine(caller)
        .build()
        .unwrap();

    runtime.call("t", nova::args!()).unwrap();
    drop(runtime);

    assert!(recorder.has_error());
    assert!(!recorder.messages().iter().any(|m| m == "a ran"), "{:?}", recorder.messages());
}

#[test]
fn merged_manifest_schemas_accept_either() {
    let recorder = Recorder::new();
    let first = routine()
        .args(object().field("name", string()))
        .step(nova::step().run("{{ info('ran') }}"))
        .build();
    let second = routine().args(object().field("count", number())).build();

    let runtime = nova::new()
        .observe(recorder.clone())
        .routine(first)
        .routine(second)
        .build()
        .unwrap();

    // merged schemas form a `oneof`: either shape is accepted.
    runtime.call("t", nova::args!(name = "x")).unwrap();
    runtime.call("t", nova::args!(count = 3)).unwrap();
    // a value matching neither shape is rejected.
    assert!(runtime.call("t", nova::args!(other = true)).is_err());
    drop(runtime);

    assert!(recorder.messages().iter().any(|m| m == "ran"), "{:?}", recorder.messages());
}

#[test]
fn schema_round_trips_through_json() {
    let schema: Schema = object().field("name", string().min(1)).into();
    let json = serde_json::to_string(&schema).unwrap();
    let back: Schema = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&back).unwrap(), json);
}
