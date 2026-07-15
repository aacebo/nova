mod common;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use common::Recorder;
use nova::event::object::{CallEvent, UpdateEvent};
use nova::event::step::{EndEvent, StartEvent};
use nova::reflect::Value;
use nova::{Binding, Event, args, event};

type ActionResult = Result<(), Box<dyn std::error::Error>>;
type FuncResult = Result<Binding, Box<dyn std::error::Error>>;

#[test]
fn listener_delivers_calls_updates_and_errors() {
    let recorder = Recorder::new();
    let runtime = nova::new()
        .observe(recorder.clone())
        .var("total", 0)
        .predicate("in_stock", |scope: &dyn nova::Context| {
            let args = scope.args();
            Ok(args.key("qty") > Value::from(0))
        })
        .func("subtotal", |scope: &dyn nova::Context| -> FuncResult {
            let args = scope.args();
            let qty = u64::try_from(args.key("qty")).unwrap_or(0);
            let unit = u64::try_from(args.key("unit")).unwrap_or(0);
            Ok(Value::from(qty * unit).into())
        })
        .action("fulfill", |scope: &dyn nova::Context| -> ActionResult {
            let args = scope.args();
            let total = nova::call!("subtotal", **args as u64);
            nova::set!("total", total);
            Ok(())
        })
        .action("reject", |scope: &dyn nova::Context| -> ActionResult {
            scope.error("out of stock");
            Ok(())
        })
        .action("process", |scope: &dyn nova::Context| -> ActionResult {
            let args = scope.args();
            if nova::call!("in_stock", **args).is_truthy() {
                nova::call!("fulfill", **args);
            } else {
                nova::call!("reject", **args);
            }
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("process", args!(qty = 3, unit = 5)).unwrap();
    runtime.call("process", args!(qty = 0, unit = 5)).unwrap();

    drop(runtime);

    let calls = recorder.calls();
    assert!(calls.contains(&"process".to_string()), "{calls:?}");
    assert!(calls.contains(&"fulfill".to_string()), "{calls:?}");
    assert!(calls.contains(&"subtotal".to_string()), "{calls:?}");
    assert!(calls.contains(&"reject".to_string()), "{calls:?}");

    let updates = recorder.updates();
    let total_update = updates
        .iter()
        .find(|(name, ..)| name == "total")
        .expect("total should emit an update");
    assert_eq!(total_update.1, Value::from(0), "old value");
    assert_eq!(total_update.2, Value::from(15u64), "new value");

    let messages = recorder.messages();
    assert!(messages.iter().any(|e| e.contains("out of stock")), "{messages:?}");
}

#[test]
fn closure_observer_receives_events() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = seen.clone();
    let runtime = nova::new()
        .observe(event::on_call(move |event: &CallEvent| {
            sink.lock().unwrap().push(event.name.clone());
        }))
        .action("noop", |_scope: &dyn nova::Context| -> ActionResult { Ok(()) })
        .build()
        .unwrap();

    runtime.call("noop", args!()).unwrap();
    drop(runtime);

    assert_eq!(*seen.lock().unwrap(), vec!["noop".to_string()]);
}

#[test]
fn multiple_observers_each_receive_every_event() {
    let recorder = Recorder::new();
    let count = Arc::new(AtomicUsize::new(0));
    let counter = count.clone();
    let runtime = nova::new()
        .observe(recorder.clone())
        .observe(move |_event: &Event| {
            counter.fetch_add(1, Ordering::SeqCst);
        })
        .var("n", 0)
        .action("bump", |scope: &dyn nova::Context| -> ActionResult {
            nova::set!("n", 1);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("bump", args!()).unwrap();
    drop(runtime);

    assert_eq!(recorder.calls(), vec!["bump".to_string()]);
    assert_eq!(recorder.updates().len(), 1);
    assert_eq!(count.load(Ordering::SeqCst), 2, "both events reached the closure observer");
}

#[test]
fn listener_accumulates_across_multiple_calls() {
    let recorder = Recorder::new();
    let runtime = nova::new()
        .observe(recorder.clone())
        .var("n", 0)
        .action("bump", |scope: &dyn nova::Context| -> ActionResult {
            nova::set!("n", 1);
            Ok(())
        })
        .action("bump_again", |scope: &dyn nova::Context| -> ActionResult {
            nova::set!("n", 2);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("bump", args!()).unwrap();
    runtime.call("bump_again", args!()).unwrap();
    drop(runtime);

    assert_eq!(recorder.calls(), vec!["bump".to_string(), "bump_again".to_string()]);
    assert_eq!(recorder.updates().len(), 2, "one overwrite per call");
}

#[test]
fn fresh_bindings_do_not_emit_updates() {
    let recorder = Recorder::new();
    let runtime = nova::new()
        .observe(recorder.clone())
        .var("known", 1)
        .action("setup", |scope: &dyn nova::Context| -> ActionResult {
            nova::set!("known", 2);
            nova::declare!("fresh", 9);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("setup", args!()).unwrap();
    drop(runtime);

    let updated: Vec<String> = recorder.updates().into_iter().map(|(name, ..)| name).collect();
    assert!(updated.contains(&"known".to_string()), "{updated:?}");
    assert!(
        !updated.contains(&"fresh".to_string()),
        "fresh insert must not emit an update: {updated:?}"
    );
}

#[test]
fn runtime_without_observers_runs_and_drops_cleanly() {
    let runtime = nova::new()
        .var("count", 0)
        .func("fn", |_: &dyn nova::Context| -> FuncResult { Ok(Value::Null.into()) })
        .action("run", |scope: &dyn nova::Context| -> ActionResult {
            nova::set!("count", 1);
            nova::call!("fn");
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("run", args!()).unwrap();
    drop(runtime);
}

#[test]
fn runtime_with_routines_and_observer_joins_on_drop() {
    let recorder = Recorder::new();
    let manifest = nova::manifest()
        .name("flow")
        .var("greeting", "hello")
        .step(nova::step().name("say").run("{{ info(greeting ~ ' world') }}"))
        .build();

    let runtime = nova::new().observe(recorder.clone()).routine(manifest).build().unwrap();

    runtime.call("flow", args!()).unwrap();
    drop(runtime);

    assert!(recorder.calls().contains(&"flow".to_string()), "{:?}", recorder.calls());
}

#[test]
fn step_events_fire_for_routine_steps() {
    let starts = Arc::new(Mutex::new(Vec::<StartEvent>::new()));
    let ends = Arc::new(Mutex::new(Vec::<EndEvent>::new()));
    let start_sink = starts.clone();
    let end_sink = ends.clone();
    let manifest = nova::manifest()
        .name("flow")
        .var("greeting", "hello")
        .step(nova::step().name("say").run("{{ info(greeting ~ ' world') }}"))
        .step(nova::step().name("again").run("{{ info('done') }}"))
        .build();

    let runtime = nova::new()
        .observe(event::on_step_start(move |e: &StartEvent| {
            start_sink.lock().unwrap().push(e.clone())
        }))
        .observe(event::on_step_end(move |e: &EndEvent| {
            end_sink.lock().unwrap().push(e.clone())
        }))
        .routine(manifest)
        .build()
        .unwrap();

    runtime.call("flow", args!()).unwrap();
    drop(runtime);

    let starts = starts.lock().unwrap();
    let ends = ends.lock().unwrap();
    let start_names: Vec<&str> = starts.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(start_names, vec!["say", "again"], "{starts:?}");
    assert_eq!(starts.len(), 2);
    assert_eq!(ends.len(), 2);
    assert_eq!(starts[0].total, 2);
    assert_eq!(starts[1].index, 1);
    assert!(ends.iter().all(|e| e.status == event::step::Status::Ok), "{ends:?}");
}

#[test]
fn step_end_status_reflects_skip_and_shell_failure_is_a_diagnostic() {
    let recorder = Recorder::new();
    let manifest = nova::manifest()
        .name("flow")
        .var("enabled", false)
        .step(nova::step().name("ok").run("{{ info('ran') }}"))
        .step(nova::step().name("skipped").guard("enabled").run("{{ info('nope') }}"))
        .step(nova::step().name("boom").shell("exit 3"))
        .build();

    let runtime = nova::new().observe(recorder.clone()).routine(manifest).build().unwrap();

    runtime.call("flow", args!()).unwrap();
    drop(runtime);

    let ends = recorder.step_ends();
    let by_name: std::collections::HashMap<&str, event::step::Status> =
        ends.iter().map(|e| (e.name.as_str(), e.status)).collect();

    assert_eq!(by_name.get("ok"), Some(&event::step::Status::Ok), "{ends:?}");
    assert_eq!(by_name.get("skipped"), Some(&event::step::Status::Skipped), "{ends:?}");

    let messages = recorder.messages();
    assert!(messages.iter().any(|m| m.contains("shell exited")), "{messages:?}");
    assert!(recorder.has_error());
}

#[test]
fn per_variant_closure_adapter_receives_only_its_variant() {
    let call_names = Arc::new(Mutex::new(Vec::<String>::new()));
    let update_count = Arc::new(AtomicUsize::new(0));
    let names_sink = call_names.clone();
    let update_sink = update_count.clone();
    let runtime = nova::new()
        .observe(event::on_call(move |e: &CallEvent| {
            names_sink.lock().unwrap().push(e.name.clone())
        }))
        .observe(event::on_update(move |_e: &UpdateEvent| {
            update_sink.fetch_add(1, Ordering::SeqCst);
        }))
        .var("n", 0)
        .action("bump", |scope: &dyn nova::Context| -> ActionResult {
            nova::set!("n", 1);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("bump", args!()).unwrap();
    drop(runtime);

    assert_eq!(*call_names.lock().unwrap(), vec!["bump".to_string()]);
    assert_eq!(update_count.load(Ordering::SeqCst), 1);
}
