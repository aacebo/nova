#![cfg(feature = "ai")]

mod common;

use common::Recorder;
use nova::ai::AI;

fn run(recorder: &Recorder, manifest: nova::Manifest) {
    let runtime = nova::new().observe(recorder.clone()).ai().routine(manifest).build().unwrap();

    runtime.call("t", nova::args!()).unwrap();
}

fn routine() -> nova::build::ManifestBuilder {
    nova::manifest().name("t").on([nova::Trigger::Run { priority: None }])
}

#[test]
fn sentiment_rejects_non_string_text() {
    let recorder = Recorder::new();
    run(&recorder, routine().step(nova::step().run("{{ ai.sentiment(42) }}")).build());
    assert!(recorder.has_error());
}

#[test]
fn entity_extraction_rejects_non_string_text() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine().step(nova::step().run("{{ ai.entities.extract(42) }}")).build(),
    );
    assert!(recorder.has_error());
}

#[test]
fn keyword_extraction_rejects_non_string_text() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine().step(nova::step().run("{{ ai.keywords.extract(42) }}")).build(),
    );
    assert!(recorder.has_error());
}

#[test]
fn pii_extraction_rejects_non_string_text() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine().step(nova::step().run("{{ ai.pii.extract(42) }}")).build(),
    );
    assert!(recorder.has_error());
}

#[test]
fn embeddings_rejects_non_string_text() {
    let recorder = Recorder::new();
    run(&recorder, routine().step(nova::step().run("{{ ai.embeddings(42) }}")).build());
    assert!(recorder.has_error());
}

#[test]
fn summarization_rejects_non_string_text() {
    let recorder = Recorder::new();
    run(&recorder, routine().step(nova::step().run("{{ ai.summarize(42) }}")).build());
    assert!(recorder.has_error());
}

#[test]
fn sentiment_rejects_non_numeric_min_score() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run("{{ ai.sentiment('hi', min_score='high') }}"))
            .build(),
    );
    assert!(recorder.has_error());
}

// Real-inference test: downloads rust-bert models (heavy). Run manually with:
//   cargo test --features ai -- --ignored
#[test]
#[ignore]
fn sentiment_classifies_positive_text() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run("{{ info(ai.sentiment('I absolutely love this, it is wonderful')[0].label) }}"))
            .build(),
    );

    assert!(!recorder.has_error(), "{:?}", recorder.messages());
    assert!(
        recorder.messages().iter().any(|m| m == "positive"),
        "{:?}",
        recorder.messages()
    );
}
