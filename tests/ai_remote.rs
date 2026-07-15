#![cfg(feature = "ai")]

mod common;

use common::Recorder;
use httpmock::prelude::*;
use nova::AI;

fn run(recorder: &Recorder, manifest: nova::Manifest) {
    let runtime = nova::new().observe(recorder.clone()).ai().routine(manifest).build().unwrap();

    runtime.call("t", nova::args!()).unwrap();
}

fn routine() -> nova::build::ManifestBuilder {
    nova::manifest().name("t").on([nova::Trigger::Run { priority: None }])
}

/// The chat-completions shape every OpenAI-compatible provider returns.
///
/// A schema-constrained completion: the model's answer is JSON, carried as a string.
fn completion(content: serde_json::Value) -> serde_json::Value {
    text_completion(&content.to_string())
}

/// A plain completion: the model's answer is prose, carried as-is.
fn text_completion(content: &str) -> serde_json::Value {
    serde_json::json!({
        "choices": [{ "message": { "content": content } }],
    })
}

#[test]
fn remote_summarize_returns_the_same_shape_as_local() {
    let server = MockServer::start();

    // Generation is a plain completion, not a schema-constrained one: the model answers with the
    // summary itself, not a JSON envelope around it.
    let mock = server.mock(|when, then| {
        when.method(POST).path("/chat/completions");
        then.status(200).json_body(text_completion("A remote summary."));
    });

    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run(format!(
                "{{% set out = ai.summarize('the quick brown fox jumps over the lazy dog again', \
                 provider='openai', model='gpt-5', base_url='{}', api_key='test') %}}\
                 {{{{ info(out[0].value) }}}}",
                server.base_url()
            )))
            .build(),
    );

    mock.assert();
    assert!(!recorder.has_error(), "{:?}", recorder.messages());
    assert!(
        recorder.messages().iter().any(|m| m == "A remote summary."),
        "{:?}",
        recorder.messages()
    );
}

#[test]
fn remote_sentiment_returns_the_same_shape_as_local() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST).path("/chat/completions");
        then.status(200)
            .json_body(completion(serde_json::json!({ "label": "negative", "score": 0.91 })));
    });

    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run(format!(
                "{{% set out = ai.sentiment('whatever', provider='openai', model='gpt-5', \
                 base_url='{}', api_key='test') %}}{{{{ info(out[0].label) }}}}",
                server.base_url()
            )))
            .build(),
    );

    mock.assert();
    assert!(!recorder.has_error(), "{:?}", recorder.messages());
    assert!(
        recorder.messages().iter().any(|m| m == "negative"),
        "{:?}",
        recorder.messages()
    );
}

#[test]
fn remote_embeddings_return_vectors() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST).path("/embeddings");
        then.status(200).json_body(serde_json::json!({
            "data": [{ "index": 0, "embedding": [0.1, 0.2, 0.3] }],
        }));
    });

    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run(format!(
                "{{% set out = ai.embeddings('hello', provider='openai', model='text-embedding-3-small', \
                 base_url='{}', api_key='test') %}}{{{{ info(out[0].vector | length) }}}}",
                server.base_url()
            )))
            .build(),
    );

    mock.assert();
    assert!(!recorder.has_error(), "{:?}", recorder.messages());
    assert!(recorder.messages().iter().any(|m| m == "3"), "{:?}", recorder.messages());
}

/// A hosted API returns entity text, never spans, so the remote transport re-anchors them.
#[test]
fn remote_entities_reanchor_offsets_in_the_source() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST).path("/chat/completions");
        then.status(200).json_body(completion(serde_json::json!({
            "entities": [{ "label": "organization", "text": "Microsoft", "score": 0.95 }],
        })));
    });

    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run(format!(
                "{{% set found = ai.entities('Satya works at Microsoft', provider='openai', \
                 model='gpt-5', base_url='{}', api_key='test') %}}\
                 {{{{ info(found[0].text ~ ':' ~ found[0].label ~ ':' ~ found[0].spans[0].begin ~ '-' ~ found[0].spans[0].end) }}}}",
                server.base_url()
            )))
            .build(),
    );

    mock.assert();
    assert!(!recorder.has_error(), "{:?}", recorder.messages());
    // "Microsoft" sits at chars 15..24 of "Satya works at Microsoft" -- the span must be
    // re-anchored there, not invented.
    assert!(
        recorder.messages().iter().any(|m| m == "Microsoft:organization:15-24"),
        "{:?}",
        recorder.messages()
    );
}

#[test]
fn remote_auth_failure_is_reported() {
    let server = MockServer::start();

    server.mock(|when, then| {
        when.method(POST).path("/chat/completions");
        then.status(401).json_body(serde_json::json!({ "error": "bad key" }));
    });

    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run(format!(
                "{{{{ ai.sentiment('hi', provider='openai', model='gpt-5', base_url='{}', api_key='nope') }}}}",
                server.base_url()
            )))
            .build(),
    );

    assert!(recorder.has_error(), "a 401 should surface as an error");
}

/// An unknown provider must fail loudly rather than silently falling through to a default client.
#[test]
fn unknown_provider_is_rejected() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run("{{ ai.sentiment('hi', provider='anthropic', model='claude') }}"))
            .build(),
    );

    assert!(recorder.has_error(), "an unknown provider must error");
}

/// A weights host is not an inference host: asking HuggingFace to classify must not silently
/// dispatch to OpenAI.
#[test]
fn a_weights_provider_cannot_host_inference() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run("{{ ai.sentiment('hi', provider='huggingface', model='some/model') }}"))
            .build(),
    );

    assert!(recorder.has_error(), "huggingface does not host inference");
}

/// Two callers hitting the same model with different keys must not share a cached client -- the
/// cached one holds the first caller's credentials.
#[test]
fn different_api_keys_do_not_share_a_cached_client() {
    let server = MockServer::start();

    let first = server.mock(|when, then| {
        when.method(POST)
            .path("/chat/completions")
            .header("authorization", "Bearer key-a");
        then.status(200)
            .json_body(completion(serde_json::json!({ "label": "positive", "score": 0.9 })));
    });

    let second = server.mock(|when, then| {
        when.method(POST)
            .path("/chat/completions")
            .header("authorization", "Bearer key-b");
        then.status(200)
            .json_body(completion(serde_json::json!({ "label": "negative", "score": 0.9 })));
    });

    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .step(nova::step().run(format!(
                "{{{{ info(ai.sentiment('x', provider='openai', model='gpt-5', base_url='{url}', api_key='key-a')[0].label) }}}}\
                 {{{{ info(ai.sentiment('x', provider='openai', model='gpt-5', base_url='{url}', api_key='key-b')[0].label) }}}}",
                url = server.base_url()
            )))
            .build(),
    );

    assert!(!recorder.has_error(), "{:?}", recorder.messages());

    // Each key must have reached the server: a shared client would send key-a twice.
    first.assert();
    second.assert();
}
