//! Integration tests driving the public `nova` API through `Runtime` as the frontend.
//!
//! Each case builds a `Runtime` via `Builder`, registers actions/predicates/routines/vars, then
//! evaluates behaviour holistically through `runtime.invoke(...)` and observable side effects.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use nova::{Action, Args, Builder, Context, Map, Predicate, Value, builtin};

/// Action that records every set of args it was invoked with, so a test can assert on what the
/// runtime dispatched without reaching into `Context` internals.
struct Recorder(Arc<Mutex<Vec<Args>>>);

impl Action for Recorder {
    fn invoke(&self, ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
        self.0.lock().unwrap().push(ctx.args().clone());
        Ok(())
    }
}

#[test]
fn invokes_registered_action_by_name() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let runtime = Builder::new().action("bump", Recorder(calls.clone())).build();

    runtime.invoke("bump", Args::new()).unwrap();

    assert_eq!(calls.lock().unwrap().len(), 1);
}

#[test]
fn invoking_unknown_name_errors() {
    let runtime = Builder::new().build();
    assert!(runtime.invoke("missing", Args::new()).is_err());
}

#[test]
fn action_receives_the_args_it_was_invoked_with() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let runtime = Builder::new().action("echo", Recorder(calls.clone())).build();

    runtime.invoke("echo", [("n", 7)]).unwrap();

    let recorded = calls.lock().unwrap();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].get("n"), Some(&Value::from(7)));
}

#[test]
fn action_chains_into_another_action_with_fresh_args() {
    let seen = Arc::new(Mutex::new(Vec::new()));

    struct Caller;

    impl Action for Caller {
        fn invoke(&self, ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
            // the caller sees its own args, then chains with new args
            assert_eq!(ctx.args().get("n"), Some(&Value::from(1)));
            ctx.call("callee", [("n", 2)])?;
            // chaining does not disturb the caller's own args
            assert_eq!(ctx.args().get("n"), Some(&Value::from(1)));
            Ok(())
        }
    }

    let runtime = Builder::new()
        .action("caller", Caller)
        .action("callee", Recorder(seen.clone()))
        .build();

    runtime.invoke("caller", [("n", 1)]).unwrap();

    let recorded = seen.lock().unwrap();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].get("n"), Some(&Value::from(2)));
}

#[test]
fn runtime_evaluates_predicate_by_name() {
    struct Always(bool);

    impl Predicate for Always {
        fn invoke(&self, _ctx: &Context) -> Result<bool, Box<dyn std::error::Error>> {
            Ok(self.0)
        }
    }

    let runtime = Builder::new()
        .predicate("yes", Always(true))
        .predicate("no", Always(false))
        .build();

    assert!(runtime.eval("yes", Args::new()).unwrap());
    assert!(!runtime.eval("no", Args::new()).unwrap());
    assert!(runtime.eval("missing", Args::new()).is_err());
}

#[test]
fn runtime_invokes_a_map_by_name_and_returns_its_value() {
    // a Map doubles its `n` arg and returns the result
    struct Double;

    impl Map for Double {
        fn invoke(&self, ctx: &mut Context) -> Result<Option<Value>, Box<dyn std::error::Error>> {
            let n = ctx.args().get("n").and_then(|v| u64::try_from(v.clone()).ok());
            Ok(n.map(|n| Value::from(n * 2)))
        }
    }

    let runtime = Builder::new().map("double", Double).build();

    assert_eq!(runtime.map("double", [("n", 21)]).unwrap(), Some(Value::from(42)));
    // a Map that is not registered surfaces an error
    assert!(runtime.map("missing", Args::new()).is_err());
}

#[test]
fn if_builtin_dispatches_the_matching_branch() {
    static THEN: AtomicUsize = AtomicUsize::new(0);
    static ELSE: AtomicUsize = AtomicUsize::new(0);

    struct GtZero;

    impl Predicate for GtZero {
        fn invoke(&self, ctx: &Context) -> Result<bool, Box<dyn std::error::Error>> {
            Ok(ctx.args().get("n") > Some(&Value::from(0)))
        }
    }

    struct Then;

    impl Action for Then {
        fn invoke(&self, _ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
            THEN.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct Else;

    impl Action for Else {
        fn invoke(&self, _ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
            ELSE.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    THEN.store(0, Ordering::SeqCst);
    ELSE.store(0, Ordering::SeqCst);
    let runtime = Builder::new()
        .predicate("gt_zero", GtZero)
        .action("then", Then)
        .action("else", Else)
        .action("branch", builtin::If::new("gt_zero", "then").or_else("else"))
        .build();

    runtime.invoke("branch", [("n", 1)]).unwrap();
    runtime.invoke("branch", [("n", 0)]).unwrap();

    assert_eq!(THEN.load(Ordering::SeqCst), 1);
    assert_eq!(ELSE.load(Ordering::SeqCst), 1);
}

#[test]
fn not_builtin_negates_a_named_predicate() {
    struct Truthy;

    impl Predicate for Truthy {
        fn invoke(&self, _ctx: &Context) -> Result<bool, Box<dyn std::error::Error>> {
            Ok(true)
        }
    }

    let runtime = Builder::new()
        .predicate("truthy", Truthy)
        .predicate("not_truthy", builtin::Not::new("truthy"))
        .build();

    assert!(runtime.eval("truthy", Args::new()).unwrap());
    assert!(!runtime.eval("not_truthy", Args::new()).unwrap());
}

#[test]
fn routine_delegates_to_its_entrypoint() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let runtime = Builder::new()
        .action("impl", Recorder(calls.clone()))
        .routine("greet", "impl")
        .build();

    runtime.invoke("greet", [("n", 5)]).unwrap();

    let recorded = calls.lock().unwrap();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].get("n"), Some(&Value::from(5)));
}

#[test]
fn globals_and_templates_render() {
    struct Render(Arc<Mutex<String>>);

    impl Action for Render {
        fn invoke(&self, ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
            let out = ctx.env().get_template("greeting")?.render(())?;
            *self.0.lock().unwrap() = out;
            Ok(())
        }
    }

    let out = Arc::new(Mutex::new(String::new()));
    let runtime = Builder::new()
        .global("subject", "world")
        .template("greeting", "hello {{ subject }}")
        .unwrap()
        .action("render", Render(out.clone()))
        .build();

    runtime.invoke("render", Args::new()).unwrap();

    assert_eq!(*out.lock().unwrap(), "hello world");
}

#[test]
fn filters_and_functions_render() {
    struct Render(Arc<Mutex<String>>);

    impl Action for Render {
        fn invoke(&self, ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
            let out = ctx.env().get_template("t")?.render(())?;
            *self.0.lock().unwrap() = out;
            Ok(())
        }
    }

    let out = Arc::new(Mutex::new(String::new()));
    let runtime = Builder::new()
        .filter("shout", |v: String| v.to_uppercase())
        .function("brand", || "nova".to_string())
        .template("t", "{{ \"hi\" | shout }}-{{ brand() }}")
        .unwrap()
        .action("render", Render(out.clone()))
        .build();

    runtime.invoke("render", Args::new()).unwrap();

    assert_eq!(*out.lock().unwrap(), "HI-nova");
}
