mod common;

use common::Recorder;
use nova::Value;

fn build(manifest: nova::Manifest, recorder: &Recorder) -> Result<nova::Runtime, Box<dyn std::error::Error>> {
    nova::new().observe(recorder.clone()).routine(manifest).build()
}

fn routine() -> nova::build::ManifestBuilder {
    nova::manifest().name("t").on([nova::Trigger::Run { priority: None }])
}

fn schema(value: serde_json::Value) -> Value {
    Value::from_serialize(value)
}

#[test]
fn valid_kwargs_pass() {
    let recorder = Recorder::new();
    let runtime = build(
        routine()
            .args(schema(serde_json::json!({
                "type": "object",
                "properties": { "name": { "type": "string" } },
                "required": ["name"],
            })))
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
            .args(schema(serde_json::json!({
                "type": "object",
                "properties": { "name": { "type": "string" } },
                "required": ["name"],
            })))
            .step(nova::step().run("{{ info('ran') }}"))
            .build(),
        &recorder,
    )
    .unwrap();

    let result = runtime.call("t", nova::args!());
    drop(runtime);

    assert!(result.is_err());
    assert!(recorder.has_error());
    assert!(!recorder.messages().iter().any(|m| m == "ran"), "{:?}", recorder.messages());
}

#[test]
fn positional_by_index_passes_and_fails() {
    let pass_recorder = Recorder::new();
    let pass = build(
        routine()
            .args(schema(serde_json::json!({
                "type": "object",
                "properties": { "0": { "type": "number" } },
                "required": ["0"],
            })))
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
            .args(schema(serde_json::json!({
                "type": "object",
                "properties": { "0": { "type": "number" } },
                "required": ["0"],
            })))
            .step(nova::step().run("{{ info('ran') }}"))
            .build(),
        &fail_recorder,
    )
    .unwrap();
    assert!(fail.call("t", nova::args!("nope")).is_err());
    drop(fail);
    assert!(fail_recorder.has_error());
    assert!(!fail_recorder.messages().iter().any(|m| m == "ran"));
}

#[test]
fn type_mismatch_fails() {
    let recorder = Recorder::new();
    let runtime = build(
        routine()
            .args(schema(serde_json::json!({
                "type": "object",
                "properties": { "count": { "type": "number" } },
                "required": ["count"],
            })))
            .step(nova::step().run("{{ info('ran') }}"))
            .build(),
        &recorder,
    )
    .unwrap();

    assert!(runtime.call("t", nova::args!(count = "not-a-number")).is_err());
    drop(runtime);
    assert!(recorder.has_error());
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
        .args(schema(serde_json::json!({
            "type": "object",
            "properties": { "name": { "type": "string" } },
            "required": ["name"],
        })))
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
fn merged_manifest_schemas_both_apply() {
    let recorder = Recorder::new();
    let first = routine()
        .args(schema(serde_json::json!({
            "type": "object",
            "properties": { "name": { "type": "string" } },
            "required": ["name"],
        })))
        .step(nova::step().run("{{ info('ran') }}"))
        .build();
    let second = routine()
        .args(schema(serde_json::json!({
            "type": "object",
            "properties": { "count": { "type": "number" } },
            "required": ["count"],
        })))
        .build();

    let runtime = nova::new()
        .observe(recorder.clone())
        .routine(first)
        .routine(second)
        .build()
        .unwrap();

    assert!(runtime.call("t", nova::args!(name = "x")).is_err());
    runtime.call("t", nova::args!(name = "x", count = 3)).unwrap();
    drop(runtime);

    assert!(recorder.messages().iter().any(|m| m == "ran"), "{:?}", recorder.messages());
    assert!(recorder.has_error());
}

#[test]
fn bad_schema_fails_at_build() {
    let recorder = Recorder::new();
    let result = build(
        routine()
            .args(schema(serde_json::json!({ "type": "not-a-real-type" })))
            .step(nova::step().run("{{ info('ran') }}"))
            .build(),
        &recorder,
    );

    assert!(result.is_err());
}
