#![cfg(feature = "http")]

use nova::http::Http;
use nova::{KArgs, Object, Value};

fn runtime() -> nova::Runtime {
    nova::new().import(Http).unwrap().build().unwrap()
}

fn scope_with(rt: &nova::Runtime, bindings: &[(&str, Value)]) -> nova::Scope {
    let scope = rt.scope().fork("t", vec![], KArgs::new());
    for (key, value) in bindings {
        scope.set_local(*key, Object::value(value.clone()));
    }
    scope
}

#[test]
fn get_rejects_non_string_uri() {
    let rt = runtime();
    let scope = scope_with(&rt, &[]);
    assert!(scope.render_str("{{ http.get(42).status }}").is_err());
}

#[test]
fn post_rejects_non_string_uri() {
    let rt = runtime();
    let scope = scope_with(&rt, &[]);
    assert!(scope.render_str("{{ http.post(42).status }}").is_err());
}

#[test]
fn put_rejects_non_string_uri() {
    let rt = runtime();
    let scope = scope_with(&rt, &[]);
    assert!(scope.render_str("{{ http.put(42).status }}").is_err());
}

#[test]
fn patch_rejects_non_string_uri() {
    let rt = runtime();
    let scope = scope_with(&rt, &[]);
    assert!(scope.render_str("{{ http.patch(42).status }}").is_err());
}

#[test]
#[ignore = "network"]
fn get_returns_response_status() {
    let rt = runtime();
    let scope = scope_with(&rt, &[("url", Value::from("https://httpbin.org/status/200"))]);
    let out = scope.render_str("{{ http.get(url).status }}").unwrap();
    assert_eq!(out, "200");
}

#[test]
#[ignore = "network"]
fn get_exposes_body_as_text() {
    let rt = runtime();
    let scope = scope_with(&rt, &[("url", Value::from("https://httpbin.org/robots.txt"))]);
    let out = scope.render_str("{{ 'User-agent' in http.get(url).text }}").unwrap();
    assert_eq!(out, "true");
}

#[test]
#[ignore = "network"]
fn post_sends_body_and_returns_status() {
    let rt = runtime();
    let scope = scope_with(&rt, &[("url", Value::from("https://httpbin.org/post"))]);
    let out = scope
        .render_str("{{ http.post(url, 'hello').status }}|{{ 'hello' in http.post(url, 'hello').text }}")
        .unwrap();
    assert_eq!(out, "200|true");
}

#[test]
#[ignore = "network"]
fn put_sends_body_and_returns_status() {
    let rt = runtime();
    let scope = scope_with(&rt, &[("url", Value::from("https://httpbin.org/put"))]);
    let out = scope.render_str("{{ http.put(url, 'hello').status }}").unwrap();
    assert_eq!(out, "200");
}

#[test]
#[ignore = "network"]
fn patch_sends_body_and_returns_status() {
    let rt = runtime();
    let scope = scope_with(&rt, &[("url", Value::from("https://httpbin.org/patch"))]);
    let out = scope.render_str("{{ http.patch(url, 'hello').status }}").unwrap();
    assert_eq!(out, "200");
}

#[test]
#[ignore = "network"]
fn get_sends_custom_headers() {
    let rt = runtime();
    let headers = std::collections::BTreeMap::from([("X-Custom", "nova-value")]);
    let scope = scope_with(
        &rt,
        &[
            ("url", Value::from("https://httpbin.org/headers")),
            ("headers", Value::from_serialize(headers)),
        ],
    );
    let out = scope
        .render_str("{{ 'nova-value' in http.get(url, headers=headers).text }}")
        .unwrap();
    assert_eq!(out, "true");
}
