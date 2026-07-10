mod common;

use common::Recorder;

fn run(recorder: &Recorder, manifest: nova::Manifest) {
    let runtime = nova::new().observe(recorder.clone()).routine(manifest).build().unwrap();

    runtime.call("t", nova::args!()).unwrap();
}

fn routine() -> nova::build::ManifestBuilder {
    nova::manifest().name("t").on([nova::Trigger::Run { priority: None }])
}

#[test]
fn env_returns_value_when_set() {
    let key = "NOVA_TEST_ENV_SET";
    unsafe { std::env::set_var(key, "present") };

    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run("{{ info(env('NOVA_TEST_ENV_SET')) }}"))
            .build(),
    );

    unsafe { std::env::remove_var(key) };
    assert!(
        recorder.messages().iter().any(|m| m == "present"),
        "{:?}",
        recorder.messages()
    );
    assert!(!recorder.has_error());
}

#[test]
fn env_returns_undefined_when_unset() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run("{{ info('[' ~ env('NOVA_TEST_ENV_MISSING') ~ ']') }}"))
            .build(),
    );

    assert!(recorder.messages().iter().any(|m| m == "[]"), "{:?}", recorder.messages());
    assert!(!recorder.has_error());
}

#[test]
fn env_returns_default_when_unset() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run("{{ info(env('NOVA_TEST_ENV_DEFAULT', default='fallback')) }}"))
            .build(),
    );

    assert!(
        recorder.messages().iter().any(|m| m == "fallback"),
        "{:?}",
        recorder.messages()
    );
    assert!(!recorder.has_error());
}

#[test]
fn env_prefers_value_over_default_when_set() {
    let key = "NOVA_TEST_ENV_OVERRIDE";
    unsafe { std::env::set_var(key, "actual") };

    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run("{{ info(env('NOVA_TEST_ENV_OVERRIDE', default='fallback')) }}"))
            .build(),
    );

    unsafe { std::env::remove_var(key) };
    assert!(recorder.messages().iter().any(|m| m == "actual"), "{:?}", recorder.messages());
    assert!(!recorder.has_error());
}

#[test]
fn env_rejects_non_string_name() {
    let recorder = Recorder::new();
    run(&recorder, routine().step(nova::step().run("{{ env(42) }}")).build());

    assert!(recorder.has_error());
}

#[test]
fn manifest_env_binds_var_into_scope() {
    let key = "NOVA_TEST_MANIFEST_ENV";
    unsafe { std::env::set_var(key, "bound-value") };

    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .env("TOKEN", "NOVA_TEST_MANIFEST_ENV")
            .step(nova::step().run("{{ info(TOKEN) }}"))
            .build(),
    );

    unsafe { std::env::remove_var(key) };
    assert!(
        recorder.messages().iter().any(|m| m == "bound-value"),
        "{:?}",
        recorder.messages()
    );
    assert!(!recorder.has_error());
}

#[test]
fn manifest_env_leaves_symbol_undefined_when_unset() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .env("TOKEN", "NOVA_TEST_MANIFEST_ENV_MISSING")
            .step(nova::step().run("{{ info('[' ~ TOKEN ~ ']') }}"))
            .build(),
    );

    assert!(recorder.messages().iter().any(|m| m == "[]"), "{:?}", recorder.messages());
    assert!(!recorder.has_error());
}
