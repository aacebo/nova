#![cfg(feature = "http")]

use nova::args;
use nova::http::Http;

fn runtime() -> nova::Runtime {
    nova::new().import(Http).unwrap().build().unwrap()
}

#[test]
fn get_rejects_non_string_uri() {
    let rt = runtime();
    assert!(rt.func("http.get", args!(42)).is_err());
}

#[test]
#[ignore = "network"]
fn get_returns_response_status() {
    let rt = runtime();
    let res = rt.func("http.get", args!("https://httpbin.org/status/200")).unwrap();
    assert_eq!(res.get_attr("status").unwrap(), nova::Value::from(200u16));
}

#[test]
#[ignore = "network"]
fn get_exposes_body_as_text() {
    let rt = runtime();
    let res = rt.func("http.get", args!("https://httpbin.org/robots.txt")).unwrap();
    let text = res.get_attr("text").unwrap();
    assert!(text.as_str().unwrap().contains("User-agent"));
}

#[test]
fn post_rejects_non_string_uri() {
    let rt = runtime();
    assert!(rt.func("http.post", args!(42)).is_err());
}

#[test]
fn put_rejects_non_string_uri() {
    let rt = runtime();
    assert!(rt.func("http.put", args!(42)).is_err());
}

#[test]
fn patch_rejects_non_string_uri() {
    let rt = runtime();
    assert!(rt.func("http.patch", args!(42)).is_err());
}

#[test]
#[ignore = "network"]
fn post_sends_body_and_returns_status() {
    let rt = runtime();
    let res = rt.func("http.post", args!("https://httpbin.org/post", "hello")).unwrap();
    assert_eq!(res.get_attr("status").unwrap(), nova::Value::from(200u16));
    assert!(res.get_attr("text").unwrap().as_str().unwrap().contains("hello"));
}

#[test]
#[ignore = "network"]
fn put_sends_body_and_returns_status() {
    let rt = runtime();
    let res = rt.func("http.put", args!("https://httpbin.org/put", "hello")).unwrap();
    assert_eq!(res.get_attr("status").unwrap(), nova::Value::from(200u16));
    assert!(res.get_attr("text").unwrap().as_str().unwrap().contains("hello"));
}

#[test]
#[ignore = "network"]
fn patch_sends_body_and_returns_status() {
    let rt = runtime();
    let res = rt.func("http.patch", args!("https://httpbin.org/patch", "hello")).unwrap();
    assert_eq!(res.get_attr("status").unwrap(), nova::Value::from(200u16));
    assert!(res.get_attr("text").unwrap().as_str().unwrap().contains("hello"));
}

#[test]
#[ignore = "network"]
fn get_sends_custom_headers() {
    let rt = runtime();
    let headers = std::collections::BTreeMap::from([("X-Custom", "nova-value")]);
    let res = rt
        .func("http.get", args!("https://httpbin.org/headers", headers = headers))
        .unwrap();
    assert_eq!(res.get_attr("status").unwrap(), nova::Value::from(200u16));
    assert!(res.get_attr("text").unwrap().as_str().unwrap().contains("nova-value"));
}
