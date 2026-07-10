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
