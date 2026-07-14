#![cfg(feature = "http")]

use httpmock::prelude::*;
use nova::{Args, Pointer, Scope, Value};
use nova::http::Http;

type FuncResult = Result<Pointer, Box<dyn std::error::Error>>;

#[test]
fn probe_http_response_survives_eval() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET).path("/x");
        then.status(201).header("content-type", "application/json").body("{\"a\":1}");
    });

    let runtime = nova::new()
        .http()
        .var("url", server.url("/x"))
        .build()
        .unwrap();

    let scope = runtime.scope();

    // direct rust call path
    let p = scope.eval("http.get(url)").unwrap();
    println!("PROBE eval http.get(url) -> {:?}", p.value());
    println!("PROBE eval http.get(url).status -> {:?}", scope.eval("http.get(url).status").map(|p| p.value().to_string()));
    println!("PROBE eval http.get(url).text -> {:?}", scope.eval("http.get(url).text").map(|p| p.value().to_string()));
    println!("PROBE render_str -> {:?}", scope.render_str("{{ http.get(url).status }}"));
    println!("PROBE render_str text -> {:?}", scope.render_str("{{ http.get(url).text }}"));
}

#[test]
fn probe_undefined_vs_null_render() {
    let runtime = nova::new()
        .func("undef", |_a: &Args, _s: &Scope| -> FuncResult { Ok(Pointer::new(Value::Undefined)) })
        .func("nul", |_a: &Args, _s: &Scope| -> FuncResult { Ok(Pointer::new(Value::Null)) })
        .build()
        .unwrap();
    let scope = runtime.scope();
    println!("PROBE undefined render -> {:?}", scope.render_str("[{{ undef() }}]"));
    println!("PROBE null render -> {:?}", scope.render_str("[{{ nul() }}]"));
    println!("PROBE undefined is defined? -> {:?}", scope.render_str("{{ undef() is defined }}"));
    println!("PROBE null is none? -> {:?}", scope.render_str("{{ nul() is none }}"));
    println!("PROBE null default -> {:?}", scope.render_str("{{ nul() or 'fallback' }}"));
    println!("PROBE eval nul -> {:?}", scope.eval("nul()").map(|p| format!("{:?}", p.value())));
    println!("PROBE eval undef -> {:?}", scope.eval("undef()").map(|p| format!("{:?}", p.value())));
}

#[test]
fn probe_number_roundtrip() {
    let runtime = nova::new().build().unwrap();
    let scope = runtime.scope();
    for expr in ["-5", "-1", "3.5", "18446744073709551615", "0.0", "-0.5", "1 - 2"] {
        let p = scope.eval(expr).unwrap();
        println!("PROBE eval {:>22} -> {:?} / display={}", expr, p.value(), p.value());
    }
}

#[test]
fn probe_list_pointer_into_owned() {
    // Vec<T> -> Pointer creates a dynamic sequence
    let p: Pointer = vec![1_i32, 2, 3].into();
    println!("PROBE list pointer value = {:?}", p.value());
    println!("PROBE list into_owned    = {:?}", p.value().into_owned());
    println!("PROBE list is_truthy     = {:?}", p.is_truthy());
    let empty: Pointer = Vec::<i32>::new().into();
    println!("PROBE empty list truthy  = {:?}", empty.is_truthy());
    println!("PROBE serialize list     = {:?}", serde_json::to_string(&p));
}

#[test]
fn probe_scope_args_symbol() {
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let s2 = seen.clone();
    let runtime = nova::new()
        .action("show", move |_a: &Args, scope: &Scope| {
            s2.lock().unwrap().push(format!("{:?}", scope.render_str("{{ args }}")));
            s2.lock().unwrap().push(format!("{:?}", scope.eval("args")));
            Ok(())
        })
        .build()
        .unwrap();
    runtime.call("show", nova::args!(1, 2)).unwrap();
    println!("PROBE args symbol -> {:?}", seen.lock().unwrap());
}

#[test]
fn probe_schema_validate_with_dynamic_arg() {
    // routine validation: args.iter() -> Value -> into_owned() drops dynamics
    let m = nova::manifest()
        .name("r")
        .on([nova::Trigger::Run { priority: None }])
        .args(
            nova::schema::object().field("items", nova::schema::array().items(nova::schema::number())),
        )
        .step(nova::step().run("{{ info('ran') }}"))
        .build();

    let runtime = nova::new().routine(m).build().unwrap();
    let items: Pointer = vec![1_i32, 2, 3].into();
    let mut kargs = nova::KArgs::new();
    kargs.set("items", items);
    let res = runtime.call("r", Args::new(Vec::new(), kargs));
    println!("PROBE routine validate w/ dynamic seq arg -> {:?}", res.map(|_| "ok").map_err(|e| e.to_string()));
}
