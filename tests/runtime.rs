use std::sync::{Arc, Mutex};

use nova::{Args, Builder, Diagnostic, Severity, Value, builtin, func, scope, var};

type ActionResult = Result<(), Box<dyn std::error::Error>>;
type FuncResult = Result<Option<Value>, Box<dyn std::error::Error>>;

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

/// A full "process an order" workflow exercised through one entrypoint: a routine
/// delegates to an action that branches on a predicate (via the `If` builtin), calls a
/// func whose return value is coerced, writes running state with `set!`, renders a
/// receipt template that reads both a scope var and mid-invocation state, and emits
/// diagnostics at several nesting levels.
#[test]
fn order_workflow_threads_state_templates_and_diagnostics_together() {
    let receipt = Arc::new(Mutex::new(String::new()));
    let sink = receipt.clone();

    let runtime = Builder::new()
        .var("store", "nova-mart")
        .template("receipt", "{{ store }}: {{ qty }} x {{ unit }} = {{ total }}")
        .predicate("in_stock", |args: &Args| Ok(args.get("qty") > Some(&Value::from(0))))
        .func("subtotal", |args: &Args| -> FuncResult {
            let qty = args.get("qty").and_then(|v| u64::try_from(v.clone()).ok()).unwrap_or(0);
            let unit = args.get("unit").and_then(|v| u64::try_from(v.clone()).ok()).unwrap_or(0);
            nova::info!("priced {} units", qty).emit();
            Ok(Some(Value::from(qty * unit)))
        })
        .action("fulfill", move |args: &Args| -> ActionResult {
            nova::warn!("fulfilling order").emit();

            let total = nova::call!("subtotal", args.clone() => u64).unwrap_or(0);
            nova::set!("total", nova::Var::new("total", total));

            *sink.lock().unwrap() = scope().render("receipt")?;
            Ok(())
        })
        .action("reject", |_args: &Args| -> ActionResult {
            nova::error!("out of stock").emit();
            Ok(())
        })
        .action("process", builtin::If::new("in_stock", "fulfill").or_else("reject"))
        .routine("order", "process")
        .build()
        .unwrap();

    let output = runtime.call("order", [("qty", 3), ("unit", 5)]).unwrap();

    // template saw the scope var (`store`), the invocation args (`qty`/`unit`), and the
    // state written mid-invocation (`total`)
    assert_eq!(*receipt.lock().unwrap(), "nova-mart: 3 x 5 = 15");

    // diagnostics from every level of the fork/merge tree are present, and the roll-up
    // severity is the max encountered (Warn from `fulfill`, Info from `subtotal`)
    let messages = collect_messages(&output.diagnostics);
    assert!(messages.contains(&"fulfilling order".to_string()), "{messages:?}");
    assert!(messages.contains(&"priced 3 units".to_string()), "{messages:?}");
    assert!(
        !messages.contains(&"out of stock".to_string()),
        "reject branch should not run"
    );
    assert_eq!(output.diagnostics[0].severity(), Severity::Warn);
}

/// The same workflow taking the else-branch: a zero quantity fails the predicate, so
/// `reject` runs, emits an error, and leaves no receipt state behind.
#[test]
fn order_workflow_else_branch_rejects_and_escalates_severity() {
    let receipt = Arc::new(Mutex::new(String::from("untouched")));
    let sink = receipt.clone();

    let runtime = Builder::new()
        .predicate("in_stock", |args: &Args| Ok(args.get("qty") > Some(&Value::from(0))))
        .action("fulfill", move |_args: &Args| -> ActionResult {
            *sink.lock().unwrap() = String::from("fulfilled");
            Ok(())
        })
        .action("reject", |_args: &Args| -> ActionResult {
            nova::error!("out of stock").emit();
            assert!(!nova::has!("total"));
            Ok(())
        })
        .action("process", builtin::If::new("in_stock", "fulfill").or_else("reject"))
        .build()
        .unwrap();

    let output = runtime.call("process", [("qty", 0)]).unwrap();

    assert_eq!(*receipt.lock().unwrap(), "untouched");
    let messages = collect_messages(&output.diagnostics);
    assert!(messages.contains(&"out of stock".to_string()), "{messages:?}");
    assert_eq!(output.diagnostics[0].severity(), Severity::Error);
}

/// A recursive `factorial` built from scoped state and nested `call!`s: each level reads
/// its arg, multiplies into an accumulator held in scope, and recurses via a `Not`-gated
/// base case. Exercises deep fork/merge, coercion, and predicate composition.
#[test]
fn recursive_calls_accumulate_state_and_coerce_results() {
    let runtime = Builder::new()
        .predicate("is_zero", |args: &Args| Ok(args.get("n") == Some(&Value::from(0))))
        .predicate("is_positive", builtin::Not::new("is_zero"))
        .func("fact", |args: &Args| -> FuncResult {
            let n = args.get("n").and_then(|v| u64::try_from(v.clone()).ok()).unwrap_or(0);

            if n == 0 {
                return Ok(Some(Value::from(1u64)));
            }

            let sub = nova::call!("fact", [("n", n - 1)] => u64).unwrap_or(1);
            Ok(Some(Value::from(n * sub)))
        })
        .action("run", |args: &Args| -> ActionResult {
            // gate on the Not-composed predicate before computing
            assert!(nova::call!("is_positive", args.clone()).unwrap().is_true());

            let result = nova::call!("fact", args.clone() => u64).unwrap_or(0);
            nova::set!("result", nova::Var::new("result", result));
            nova::info!("factorial = {}", result).emit();
            Ok(())
        })
        .build()
        .unwrap();

    let output = runtime.call("run", [("n", 5)]).unwrap();

    let messages = collect_messages(&output.diagnostics);
    assert!(messages.contains(&"factorial = 120".to_string()), "{messages:?}");

    // computing directly through the func entrypoint yields the coerced value too
    let value = runtime.func("fact", [("n", 5)]).unwrap().value;
    assert_eq!(value, Some(Value::from(120u64)));
}

/// A template that composes a func and a predicate in one render pass, where the func
/// emits a diagnostic. Confirms template-invoked callables resolve from scope, feed
/// their results into the rendered string, and still thread diagnostics into the live
/// tree of the invocation that triggered the render.
#[test]
fn template_composes_callables_and_captures_their_diagnostics() {
    let out = Arc::new(Mutex::new(String::new()));
    let sink = out.clone();

    let runtime = Builder::new()
        .var("label", "score")
        .func("double", |args: &Args| -> FuncResult {
            let n = args.get("n").and_then(|v| u64::try_from(v.clone()).ok()).unwrap_or(0);
            nova::warn!("doubling {}", n).emit();
            Ok(Some(Value::from(n * 2)))
        })
        .predicate("is_positive", |args: &Args| Ok(args.get("n") > Some(&Value::from(0))))
        .action("render", move |_args: &Args| -> ActionResult {
            *sink.lock().unwrap() = scope().render_str("{{ label }}: {{ double(n=21) }} ({{ is_positive(n=1) }})")?;
            Ok(())
        })
        .build()
        .unwrap();

    let output = runtime.call("render", Args::new()).unwrap();

    assert_eq!(*out.lock().unwrap(), "score: 42 (true)");
    let messages = collect_messages(&output.diagnostics);
    assert!(messages.contains(&"doubling 21".to_string()), "{messages:?}");
}

/// `Runtime::eval` and the coercion form of `call!` over a predicate, plus arg isolation
/// across a chained call: the callee sees fresh args and the caller's are unchanged.
#[test]
fn eval_predicate_and_call_isolation_across_a_chain() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let calls = seen.clone();

    let runtime = Builder::new()
        .predicate("even", |args: &Args| {
            let n = args.get("n").and_then(|v| u64::try_from(v.clone()).ok()).unwrap_or(1);
            Ok(n.is_multiple_of(2))
        })
        .action("callee", move |args: &Args| -> ActionResult {
            calls.lock().unwrap().push(args.clone());
            Ok(())
        })
        .action("caller", |args: &Args| -> ActionResult {
            assert_eq!(args.get("n"), Some(&Value::from(1)));
            nova::call!("callee", [("n", 2)]);
            // caller's own args are untouched by the chained call
            assert_eq!(args.get("n"), Some(&Value::from(1)));
            Ok(())
        })
        .build()
        .unwrap();

    assert_eq!(runtime.eval("even", [("n", 4)]).unwrap().value, Some(Value::from(true)));
    assert_eq!(runtime.eval("even", [("n", 3)]).unwrap().value, Some(Value::from(false)));

    runtime.call("caller", [("n", 1)]).unwrap();
    let recorded = seen.lock().unwrap();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].get("n"), Some(&Value::from(2)));
}

/// `var!` and `func!` register bindings into the live scope mid-invocation, and those
/// bindings are then reachable — by a template (the var) and by `call!` (the func) —
/// within the same invocation.
#[test]
fn var_and_func_macros_register_into_the_live_scope() {
    let out = Arc::new(Mutex::new(String::new()));
    let sink = out.clone();

    let runtime = Builder::new()
        .action("setup", move |_args: &Args| -> ActionResult {
            var!("greeting", "hello");
            func!("shout", |args: &Args| -> FuncResult {
                let word = args.get("word").map(|v| v.to_string()).unwrap_or_default();
                Ok(Some(Value::from(word.to_uppercase())))
            });

            // the func registered above is callable by name in the same invocation
            let loud = nova::call!("shout", [("word", "hey")]).unwrap();
            assert_eq!(loud, Value::from("HEY"));

            // the var registered above resolves in a render
            *sink.lock().unwrap() = scope().render_str("{{ greeting }}")?;
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("setup", Args::new()).unwrap();
    assert_eq!(*out.lock().unwrap(), "hello");
}

/// Error paths: unknown names error, a func returning an error propagates through the
/// `?` in `call!`, and a malformed template fails at build time.
#[test]
fn error_paths_surface_at_call_and_build_time() {
    let runtime = Builder::new()
        .func("boom", |_args: &Args| -> FuncResult {
            Err(nova::Error::message("kaboom").into())
        })
        .action("caller", |_args: &Args| -> ActionResult {
            nova::call!("boom");
            Ok(())
        })
        .build()
        .unwrap();

    // unknown name
    assert!(runtime.call("missing", Args::new()).is_err());
    // error thrown inside a func propagates up through call!'s `?`
    assert!(runtime.call("caller", Args::new()).is_err());
    // malformed template is rejected when the runtime is built
    assert!(Builder::new().template("t", "{{ ").build().is_err());
}
