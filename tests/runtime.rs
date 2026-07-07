use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use nova::{Args, Builder, Context, Diagnostic, Severity, Value, builtin};

type ActionResult = Result<(), Box<dyn std::error::Error>>;
type FuncResult = Result<Option<Value>, Box<dyn std::error::Error>>;

/// An action closure that records the args it was invoked with.
fn recorder(calls: Arc<Mutex<Vec<Args>>>) -> impl Fn(&mut Context) -> ActionResult {
    move |ctx: &mut Context| {
        calls.lock().unwrap().push(ctx.args().clone());
        Ok(())
    }
}

/// An action closure that renders `source` into `out`.
fn render_into(out: Arc<Mutex<String>>, source: &'static str) -> impl Fn(&mut Context) -> ActionResult {
    move |ctx: &mut Context| {
        *out.lock().unwrap() = ctx.render_str(source)?;
        Ok(())
    }
}

#[test]
fn invokes_registered_action_by_name() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let runtime = Builder::new().action("bump", recorder(calls.clone())).build().unwrap();

    runtime.call("bump", Args::new()).unwrap();
    assert_eq!(calls.lock().unwrap().len(), 1);
}

#[test]
fn diagnostics_emitted_by_actions_propagate_to_the_frontend() {
    let runtime = Builder::new()
        .action("parent", |ctx: &mut Context| -> ActionResult {
            ctx.emit(nova::warn!("from parent"));
            ctx.call("child", Args::new())?;
            Ok(())
        })
        .action("child", |ctx: &mut Context| -> ActionResult {
            ctx.emit(nova::info!("from child"));
            Ok(())
        })
        .build()
        .unwrap();

    let output = runtime.call("parent", Args::new()).unwrap();

    assert_eq!(output.value, None);
    assert_eq!(output.diagnostics.len(), 1);

    let parent_node = &output.diagnostics[0];

    assert_eq!(parent_node.message.as_deref(), Some("parent"));
    assert_eq!(parent_node.severity(), Severity::Warn);
    assert_eq!(parent_node.children.len(), 2);
    assert_eq!(parent_node.children[0].message.as_deref(), Some("from parent"));

    let child_node = &parent_node.children[1];

    assert_eq!(child_node.message.as_deref(), Some("child"));
    assert_eq!(child_node.severity(), Severity::Info);
    assert_eq!(child_node.children.len(), 1);
    assert_eq!(child_node.children[0].message.as_deref(), Some("from child"));
}

#[test]
fn invoking_unknown_name_errors() {
    let runtime = Builder::new().build().unwrap();
    assert!(runtime.call("missing", Args::new()).is_err());
}

#[test]
fn action_receives_the_args_it_was_invoked_with() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let runtime = Builder::new().action("echo", recorder(calls.clone())).build().unwrap();

    runtime.call("echo", [("n", 7)]).unwrap();
    let recorded = calls.lock().unwrap();

    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].get("n"), Some(&Value::from(7)));
}

#[test]
fn action_chains_into_another_action_with_fresh_args() {
    let seen = Arc::new(Mutex::new(Vec::new()));

    let runtime = Builder::new()
        .action("caller", |ctx: &mut Context| -> ActionResult {
            assert_eq!(ctx.args().get("n"), Some(&Value::from(1)));
            ctx.call("callee", [("n", 2)])?;
            assert_eq!(ctx.args().get("n"), Some(&Value::from(1)));
            Ok(())
        })
        .action("callee", recorder(seen.clone()))
        .build()
        .unwrap();

    runtime.call("caller", [("n", 1)]).unwrap();
    let recorded = seen.lock().unwrap();

    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].get("n"), Some(&Value::from(2)));
}

#[test]
fn runtime_evaluates_predicate_by_name() {
    let runtime = Builder::new()
        .predicate("yes", |_ctx: &Context| Ok(true))
        .predicate("no", |_ctx: &Context| Ok(false))
        .build()
        .unwrap();

    assert_eq!(runtime.eval("yes", Args::new()).unwrap().value, Some(Value::from(true)));
    assert_eq!(runtime.eval("no", Args::new()).unwrap().value, Some(Value::from(false)));
    assert!(runtime.eval("missing", Args::new()).is_err());
}

#[test]
fn runtime_invokes_a_func_by_name_and_returns_its_value() {
    let runtime = Builder::new()
        .func("double", |ctx: &mut Context| -> FuncResult {
            let n = ctx.args().get("n").and_then(|v| u64::try_from(v.clone()).ok());
            Ok(n.map(|n| Value::from(n * 2)))
        })
        .build()
        .unwrap();

    let output = runtime.func("double", [("n", 21)]).unwrap();

    assert_eq!(output.value, Some(Value::from(42)));
    assert!(output.diagnostics.is_empty());
    assert!(runtime.func("missing", Args::new()).is_err());
}

#[test]
fn if_builtin_dispatches_the_matching_branch() {
    static THEN: AtomicUsize = AtomicUsize::new(0);
    static ELSE: AtomicUsize = AtomicUsize::new(0);

    THEN.store(0, Ordering::SeqCst);
    ELSE.store(0, Ordering::SeqCst);
    let runtime = Builder::new()
        .predicate("gt_zero", |ctx: &Context| Ok(ctx.args().get("n") > Some(&Value::from(0))))
        .action("then", |_ctx: &mut Context| -> ActionResult {
            THEN.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .action("else", |_ctx: &mut Context| -> ActionResult {
            ELSE.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .action("branch", builtin::If::new("gt_zero", "then").or_else("else"))
        .build()
        .unwrap();

    runtime.call("branch", [("n", 1)]).unwrap();
    runtime.call("branch", [("n", 0)]).unwrap();
    assert_eq!(THEN.load(Ordering::SeqCst), 1);
    assert_eq!(ELSE.load(Ordering::SeqCst), 1);
}

#[test]
fn not_builtin_negates_a_named_predicate() {
    let runtime = Builder::new()
        .predicate("truthy", |_ctx: &Context| Ok(true))
        .predicate("not_truthy", builtin::Not::new("truthy"))
        .build()
        .unwrap();

    assert_eq!(runtime.eval("truthy", Args::new()).unwrap().value, Some(Value::from(true)));
    assert_eq!(
        runtime.eval("not_truthy", Args::new()).unwrap().value,
        Some(Value::from(false))
    );
}

#[test]
fn routine_delegates_to_its_entrypoint() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let runtime = Builder::new()
        .action("impl", recorder(calls.clone()))
        .routine("greet", "impl")
        .build()
        .unwrap();

    runtime.call("greet", [("n", 5)]).unwrap();
    let recorded = calls.lock().unwrap();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].get("n"), Some(&Value::from(5)));
}

#[test]
fn registered_template_renders_with_scope_vars() {
    let out = Arc::new(Mutex::new(String::new()));
    let subject = out.clone();
    let runtime = Builder::new()
        .var("subject", "world")
        .template("greeting", "hello {{ subject }}")
        .action("render", move |ctx: &mut Context| -> ActionResult {
            *subject.lock().unwrap() = ctx.render("greeting")?;
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("render", Args::new()).unwrap();
    assert_eq!(*out.lock().unwrap(), "hello world");
}

#[test]
fn malformed_template_errors_at_build() {
    assert!(Builder::new().template("t", "{{ ").build().is_err());
}

#[test]
fn template_reads_invocation_arg_as_raw_ident() {
    let out = Arc::new(Mutex::new(String::new()));
    let runtime = Builder::new()
        .action("render", render_into(out.clone(), "n is {{ n }}"))
        .build()
        .unwrap();

    runtime.call("render", [("n", 5)]).unwrap();
    assert_eq!(*out.lock().unwrap(), "n is 5");
}

#[test]
fn template_reads_scope_var_by_bare_name() {
    let out = Arc::new(Mutex::new(String::new()));
    let runtime = Builder::new()
        .var("subject", "world")
        .action("render", render_into(out.clone(), "hello {{ subject }}"))
        .build()
        .unwrap();

    runtime.call("render", Args::new()).unwrap();
    assert_eq!(*out.lock().unwrap(), "hello world");
}

#[test]
fn template_sees_var_set_mid_invocation() {
    let out = Arc::new(Mutex::new(String::new()));
    let sink = out.clone();
    let runtime = Builder::new()
        .action("render", move |ctx: &mut Context| -> ActionResult {
            ctx.set("mood", nova::Var::new("mood", "great"));
            *sink.lock().unwrap() = ctx.render_str("feeling {{ mood }}")?;
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("render", Args::new()).unwrap();
    assert_eq!(*out.lock().unwrap(), "feeling great");
}

#[test]
fn template_can_call_a_func_from_scope() {
    let out = Arc::new(Mutex::new(String::new()));
    let runtime = Builder::new()
        .func("double", |ctx: &mut Context| -> FuncResult {
            let n = ctx.args().get("n").and_then(|v| u64::try_from(v.clone()).ok());
            Ok(n.map(|n| Value::from(n * 2)))
        })
        .action("render", render_into(out.clone(), "{{ double(n=21) }}"))
        .build()
        .unwrap();

    runtime.call("render", Args::new()).unwrap();
    assert_eq!(*out.lock().unwrap(), "42");
}

#[test]
fn template_can_call_a_predicate_from_scope() {
    let out = Arc::new(Mutex::new(String::new()));
    let runtime = Builder::new()
        .predicate("is_positive", |ctx: &Context| Ok(ctx.args().get("n") > Some(&Value::from(0))))
        .action(
            "render",
            render_into(out.clone(), "{{ is_positive(n=1) }}/{{ is_positive(n=-1) }}"),
        )
        .build()
        .unwrap();

    runtime.call("render", Args::new()).unwrap();
    assert_eq!(*out.lock().unwrap(), "true/false");
}

#[test]
fn func_called_from_template_emits_into_the_live_tree() {
    let runtime = Builder::new()
        .func("noisy", |ctx: &mut Context| -> FuncResult {
            ctx.emit(nova::warn!("from func"));
            Ok(Some(Value::from(1)))
        })
        .action("render", render_into(Arc::new(Mutex::new(String::new())), "{{ noisy() }}"))
        .build()
        .unwrap();

    let output = runtime.call("render", Args::new()).unwrap();

    let messages: Vec<_> = collect_messages(&output.diagnostics);
    assert!(messages.contains(&"from func".to_string()), "diagnostics: {messages:?}");
}

#[test]
fn macros_thread_the_invocation_trace_id_without_a_ctx_arg() {
    let runtime = Builder::new()
        .action("parent", |ctx: &mut Context| -> ActionResult {
            ctx.emit(nova::warn!("from parent"));
            ctx.call("child", Args::new())?;
            Ok(())
        })
        .action("child", |ctx: &mut Context| -> ActionResult {
            ctx.emit(nova::error!("from child"));
            Ok(())
        })
        .build()
        .unwrap();

    let output = runtime.call("parent", Args::new()).unwrap();

    let parent_node = &output.diagnostics[0];
    assert_eq!(parent_node.message.as_deref(), Some("parent"));

    let parent_diag = &parent_node.children[0];
    assert_eq!(parent_diag.message.as_deref(), Some("from parent"));

    let child_diag = &parent_node.children[1].children[0];
    assert_eq!(child_diag.message.as_deref(), Some("from child"));

    assert_eq!(parent_diag.trace_id, output.trace_id);
    assert_eq!(child_diag.trace_id, output.trace_id);
    assert_eq!(parent_node.severity(), Severity::Error);
}

fn collect_messages(diagnostics: &[Diagnostic]) -> Vec<String> {
    let mut out = Vec::new();
    for d in diagnostics {
        if let Some(m) = &d.message {
            out.push(m.clone());
        }
        out.extend(collect_messages(&d.children));
    }
    out
}
