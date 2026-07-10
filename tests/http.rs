#![cfg(feature = "http")]

mod common;

use std::collections::BTreeMap;

use common::Recorder;
use httpmock::prelude::*;
use nova::http::Http;

fn run(recorder: &Recorder, manifest: nova::Manifest) {
    let runtime = nova::new()
        .observe(recorder.clone())
        .http()
        .routine(manifest)
        .build()
        .unwrap();

    runtime.call("t", nova::args!()).unwrap();
}

fn routine() -> nova::build::ManifestBuilder {
    nova::manifest().name("t").on([nova::Trigger::Run { priority: None }])
}

#[test]
fn get_rejects_non_string_uri() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine().step(nova::step().run("{{ http.get(42).status }}")).build(),
    );
    assert!(recorder.has_error());
}

#[test]
fn post_rejects_non_string_uri() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine().step(nova::step().run("{{ http.post(42).status }}")).build(),
    );
    assert!(recorder.has_error());
}

#[test]
fn put_rejects_non_string_uri() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine().step(nova::step().run("{{ http.put(42).status }}")).build(),
    );
    assert!(recorder.has_error());
}

#[test]
fn patch_rejects_non_string_uri() {
    let recorder = Recorder::new();
    run(
        &recorder,
        routine().step(nova::step().run("{{ http.patch(42).status }}")).build(),
    );
    assert!(recorder.has_error());
}

#[test]
fn get_returns_response_status() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/status");
        then.status(200);
    });

    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .var("url", server.url("/status"))
            .step(nova::step().run("{{ info(http.get(url).status) }}"))
            .build(),
    );

    mock.assert();
    assert!(recorder.messages().iter().any(|m| m == "200"), "{:?}", recorder.messages());
}

#[test]
fn get_exposes_body_as_text() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/robots");
        then.status(200).body("User-agent: *");
    });

    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .var("url", server.url("/robots"))
            .step(nova::step().run("{{ info('User-agent' in http.get(url).text) }}"))
            .build(),
    );

    mock.assert();
    assert!(recorder.messages().iter().any(|m| m == "true"), "{:?}", recorder.messages());
}

#[test]
fn post_sends_body_and_returns_status() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/submit").body("hello");
        then.status(200).body("ok");
    });

    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .var("url", server.url("/submit"))
            .step(nova::step().run("{{ info(http.post(url, 'hello').status ~ '|' ~ http.post(url, 'hello').text) }}"))
            .build(),
    );

    mock.assert_calls(2);
    assert!(recorder.messages().iter().any(|m| m == "200|ok"), "{:?}", recorder.messages());
}

#[test]
fn put_sends_body_and_returns_status() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(PUT).path("/resource").body("hello");
        then.status(200);
    });

    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .var("url", server.url("/resource"))
            .step(nova::step().run("{{ info(http.put(url, 'hello').status) }}"))
            .build(),
    );

    mock.assert();
    assert!(recorder.messages().iter().any(|m| m == "200"), "{:?}", recorder.messages());
}

#[test]
fn patch_sends_body_and_returns_status() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(PATCH).path("/resource").body("hello");
        then.status(200);
    });

    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .var("url", server.url("/resource"))
            .step(nova::step().run("{{ info(http.patch(url, 'hello').status) }}"))
            .build(),
    );

    mock.assert();
    assert!(recorder.messages().iter().any(|m| m == "200"), "{:?}", recorder.messages());
}

#[test]
fn get_sends_custom_headers() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/headers").header("X-Custom", "nova-value");
        then.status(200);
    });

    let recorder = Recorder::new();
    run(
        &recorder,
        routine()
            .var("url", server.url("/headers"))
            .var(
                "headers",
                nova::Value::from_serialize(BTreeMap::from([("X-Custom", "nova-value")])),
            )
            .step(nova::step().run("{{ info(http.get(url, headers=headers).status) }}"))
            .build(),
    );

    mock.assert();
    assert!(recorder.messages().iter().any(|m| m == "200"), "{:?}", recorder.messages());
}
