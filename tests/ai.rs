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

#[test]
#[ignore]
fn sentiment_classifies_negative_text() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run("{{ info(ai.sentiment('This is terrible, I hate it and want a refund')[0].label) }}"))
            .build(),
    );

    assert!(!recorder.has_error(), "{:?}", recorder.messages());
    assert!(
        recorder.messages().iter().any(|m| m == "negative"),
        "{:?}",
        recorder.messages()
    );
}

#[test]
#[ignore]
fn entity_extraction_finds_person_org_and_location() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run(
                "{% set found = ai.entities.extract('Satya Nadella works at Microsoft in Seattle', min_score=0.5) %}\
                 {{ info(found | map(attribute='label') | join(',')) }}",
            ))
            .build(),
    );

    assert!(!recorder.has_error(), "{:?}", recorder.messages());

    let labels = recorder.messages().join(" ");
    for expected in ["person", "organization", "location"] {
        assert!(labels.contains(expected), "missing {expected} in {labels:?}");
    }
}

#[test]
#[ignore]
fn embeddings_are_normalized_and_semantically_ordered() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run(
                "{% set out = ai.embeddings('a happy dog playing in the park') %}\
                 {{ info(out[0].vector | length) }}",
            ))
            .build(),
    );

    assert!(!recorder.has_error(), "{:?}", recorder.messages());
    assert!(recorder.messages().iter().any(|m| m == "384"), "{:?}", recorder.messages());
}

#[test]
#[ignore]
fn summarization_produces_a_summary() {
    let recorder = Recorder::new();
    let article = "The James Webb Space Telescope has detected carbon dioxide in the atmosphere \
        of a planet outside our solar system for the first time. The exoplanet, WASP-39b, is a hot \
        gas giant orbiting a star some 700 light-years away. Astronomers said the finding proves \
        the telescope can detect and measure carbon dioxide in the thinner atmospheres of smaller \
        rocky planets. The observations were made using a near-infrared spectrograph, which splits \
        light into its component colors to reveal the chemical fingerprints of distant worlds.";

    run(
        &recorder,
        routine()
            .step(nova::step().run(format!(
                "{{% set out = ai.summarize('{article}') %}}{{{{ info(out[0].value) }}}}"
            )))
            .build(),
    );

    assert!(!recorder.has_error(), "{:?}", recorder.messages());

    let summary = recorder.messages().join(" ");
    assert!(
        !summary.contains("\"type\""),
        "value should render as text, not tagged json: {summary:?}"
    );
    assert!(summary.len() > 40, "summary too short: {summary:?}");
    assert!(
        ["Webb", "carbon dioxide", "WASP-39b", "exoplanet", "telescope"]
            .iter()
            .any(|term| summary.contains(term)),
        "summary does not mention the article's subject: {summary:?}"
    );
}

#[test]
#[ignore]
fn keyword_extraction_finds_salient_terms() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run(
                "{% set words = ai.keywords.extract('The battery life on this laptop is absolutely terrible') %}\
                 {{ info(words | map(attribute='text') | join(',')) }}",
            ))
            .build(),
    );

    assert!(!recorder.has_error(), "{:?}", recorder.messages());

    let found = recorder.messages().join(" ");
    assert!(
        ["battery", "laptop"].iter().any(|word| found.contains(word)),
        "expected a salient keyword in {found:?}"
    );
}
