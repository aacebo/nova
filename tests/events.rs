use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use nova::event::object::{CallEvent, UpdateEvent};
use nova::event::step::{EndEvent, StartEvent};
use nova::{Event, KArgs, Object, Observer, Scope, Value, event};

type ActionResult = Result<(), Box<dyn std::error::Error>>;
type FuncResult = Result<Option<Value>, Box<dyn std::error::Error>>;

#[derive(Clone, Default)]
struct Recorder(Arc<RecorderInner>);

#[derive(Default)]
struct RecorderInner {
    calls: Mutex<Vec<String>>,
    updates: Mutex<Vec<(String, Value, Value)>>,
    errors: Mutex<Vec<String>>,
}

impl Recorder {
    fn calls(&self) -> Vec<String> {
        self.0.calls.lock().unwrap().clone()
    }

    fn updates(&self) -> Vec<(String, Value, Value)> {
        self.0.updates.lock().unwrap().clone()
    }

    fn errors(&self) -> Vec<String> {
        self.0.errors.lock().unwrap().clone()
    }
}

impl Observer for Recorder {
    fn on_call(&self, event: &CallEvent) {
        self.0.calls.lock().unwrap().push(event.name.clone());
    }

    fn on_update(&self, event: &UpdateEvent) {
        self.0
            .updates
            .lock()
            .unwrap()
            .push((event.name.clone(), event.from.clone(), event.to.clone()));
    }

    fn on_error(&self, event: &nova::Error) {
        self.0.errors.lock().unwrap().push(event.to_string());
    }
}

#[test]
fn listener_delivers_calls_updates_and_errors() {
    let recorder = Recorder::default();
    let runtime = nova::new()
        .observe(recorder.clone())
        .var("total", 0)
        .predicate("in_stock", |_args: &[Value], kargs: &KArgs, _scope: &Scope| {
            Ok(kargs.get("qty") > Some(&Value::from(0)))
        })
        .func("subtotal", |_args: &[Value], kargs: &KArgs, _scope: &Scope| -> FuncResult {
            let qty = kargs.get("qty").and_then(|v| u64::try_from(v.clone()).ok()).unwrap_or(0);
            let unit = kargs.get("unit").and_then(|v| u64::try_from(v.clone()).ok()).unwrap_or(0);
            Ok(Some(Value::from(qty * unit)))
        })
        .action("fulfill", |args: &[Value], kargs: &KArgs, scope: &Scope| -> ActionResult {
            let total = nova::call!("subtotal", *args, **kargs as u64).unwrap_or(0);
            nova::set!("total", total);
            Ok(())
        })
        .action("reject", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            scope.error("out of stock");
            Ok(())
        })
        .action("process", |args: &[Value], kargs: &KArgs, scope: &Scope| -> ActionResult {
            if nova::call!("in_stock", *args, **kargs).map(|v| v.is_true()).unwrap_or(false) {
                nova::call!("fulfill", *args, **kargs);
            } else {
                nova::call!("reject", *args, **kargs);
            }
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("process", [("qty", 3), ("unit", 5)]).unwrap();
    runtime.call("process", [("qty", 0), ("unit", 5)]).unwrap();

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

    let errors = recorder.errors();
    assert!(errors.iter().any(|e| e.contains("out of stock")), "{errors:?}");
}

#[test]
fn closure_observer_receives_events() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = seen.clone();
    let runtime = nova::new()
        .observe(event::on_call(move |event: &CallEvent| {
            sink.lock().unwrap().push(event.name.clone());
        }))
        .action("noop", |_args: &[Value], _kargs: &KArgs, _scope: &Scope| -> ActionResult {
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("noop", KArgs::new()).unwrap();
    drop(runtime);

    assert_eq!(*seen.lock().unwrap(), vec!["noop".to_string()]);
}

#[test]
fn multiple_observers_each_receive_every_event() {
    let recorder = Recorder::default();
    let count = Arc::new(AtomicUsize::new(0));
    let counter = count.clone();
    let runtime = nova::new()
        .observe(recorder.clone())
        .observe(move |_event: &Event| {
            counter.fetch_add(1, Ordering::SeqCst);
        })
        .var("n", 0)
        .action("bump", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            nova::set!("n", 1);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("bump", KArgs::new()).unwrap();
    drop(runtime);

    assert_eq!(recorder.calls(), vec!["bump".to_string()]);
    assert_eq!(recorder.updates().len(), 1);
    assert_eq!(count.load(Ordering::SeqCst), 2, "both events reached the closure observer");
}

#[test]
fn listener_accumulates_across_multiple_calls() {
    let recorder = Recorder::default();
    let runtime = nova::new()
        .observe(recorder.clone())
        .var("n", 0)
        .action("bump", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            nova::set!("n", 1);
            Ok(())
        })
        .action(
            "bump_again",
            |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
                nova::set!("n", 2);
                Ok(())
            },
        )
        .build()
        .unwrap();

    runtime.call("bump", KArgs::new()).unwrap();
    runtime.call("bump_again", KArgs::new()).unwrap();
    drop(runtime);

    assert_eq!(recorder.calls(), vec!["bump".to_string(), "bump_again".to_string()]);
    assert_eq!(recorder.updates().len(), 2, "one overwrite per call");
}

#[test]
fn fresh_bindings_do_not_emit_updates() {
    let recorder = Recorder::default();
    let runtime = nova::new()
        .observe(recorder.clone())
        .var("known", 1)
        .action("setup", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            nova::set!("known", 2);
            nova::set!("fresh", 9);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("setup", KArgs::new()).unwrap();
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
        .action("run", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            nova::set!("count", 1);
            scope.set(
                "fn",
                Object::func("noop", |_: &[Value], _: &KArgs, _: &Scope| -> FuncResult { Ok(None) }),
            );
            nova::call!("fn");
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("run", KArgs::new()).unwrap();
    drop(runtime);
}

#[test]
fn runtime_with_routines_and_observer_joins_on_drop() {
    let recorder = Recorder::default();
    let manifest = nova::manifest()
        .name("flow")
        .var("greeting", "hello")
        .step(nova::step().name("say").run("{{ info(greeting ~ ' world') }}"))
        .build();

    let runtime = nova::new().observe(recorder.clone()).routine(manifest).build().unwrap();

    runtime.call("flow", KArgs::new()).unwrap();
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

    runtime.call("flow", KArgs::new()).unwrap();
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
        .action("bump", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            nova::set!("n", 1);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("bump", KArgs::new()).unwrap();
    drop(runtime);

    assert_eq!(*call_names.lock().unwrap(), vec!["bump".to_string()]);
    assert_eq!(update_count.load(Ordering::SeqCst), 1);
}
