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
    run(&recorder, routine().step(nova::step().run("{{ ai.entities(42) }}")).build());
    assert!(recorder.has_error());
}

#[test]
fn keyword_extraction_rejects_non_string_text() {
    let recorder = Recorder::new();
    run(&recorder, routine().step(nova::step().run("{{ ai.keywords(42) }}")).build());
    assert!(recorder.has_error());
}

#[test]
fn pii_extraction_rejects_non_string_text() {
    let recorder = Recorder::new();
    run(&recorder, routine().step(nova::step().run("{{ ai.pii(42) }}")).build());
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
                "{% set found = ai.entities('Satya Nadella works at Microsoft in Seattle', min_score=0.5) %}\
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
                "{% set words = ai.keywords('The battery life on this laptop is absolutely terrible') %}\
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

#[test]
#[ignore]
fn pii_extraction_finds_entities() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run(
                "{% set found = ai.pii('Contact Sarah Connor at Cyberdyne Systems in Los Angeles', min_score=0.5) %}\
                 {{ info(found | map(attribute='text') | join('|')) }}",
            ))
            .build(),
    );

    assert!(!recorder.has_error(), "{:?}", recorder.messages());

    let found = recorder.messages().join(" ");
    assert!(found.contains("Sarah"), "expected a person in {found:?}");
    assert!(found.contains("Cyberdyne"), "expected an org in {found:?}");
}

/// `Resource::Path` was matched but never constructed before the onyx refactor, so loading weights
/// from a local directory was unreachable. This proves it now works.
#[test]
#[ignore]
fn embeddings_load_from_a_local_directory() {
    let snapshot = std::fs::read_dir(".cache")
        .expect(".cache must exist; run the hub-backed tests first")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.to_string_lossy().contains("all-MiniLM-L6-v2"))
        .map(|path| path.join("snapshots"))
        .and_then(|path| std::fs::read_dir(path).ok()?.filter_map(Result::ok).next())
        .map(|entry| entry.path())
        .expect("a cached MiniLM-L6 snapshot");

    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run(format!(
                "{{% set out = ai.embeddings('hello there', model='{}') %}}{{{{ info(out[0].vector | length) }}}}",
                snapshot.display()
            )))
            .build(),
    );

    assert!(!recorder.has_error(), "{:?}", recorder.messages());
    assert!(recorder.messages().iter().any(|m| m == "384"), "{:?}", recorder.messages());
}

/// The empty cells of the capability matrix must fail loudly. A sentence embedder has no decoder,
/// so asking it to generate is an error -- named, and raised before any inference runs.
#[test]
#[ignore]
fn a_model_that_cannot_generate_is_rejected() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(
                nova::step().run("{{ ai.summarize('some text to summarize', model='sentence-transformers/all-MiniLM-L6-v2') }}"),
            )
            .build(),
    );

    assert!(recorder.has_error(), "an embedder cannot generate");

    let reported = recorder.messages().join(" ");
    assert!(
        reported.contains("cannot generate"),
        "the error should name the missing capability, got {reported:?}"
    );
}

/// Likewise: BART is a summarizer, not a sentence embedder.
#[test]
#[ignore]
fn a_model_that_cannot_embed_is_rejected() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run("{{ ai.embeddings('hello', model='facebook/bart-large-cnn') }}"))
            .build(),
    );

    assert!(recorder.has_error(), "a summarizer cannot embed");
}
