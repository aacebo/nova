use std::sync::{Arc, Mutex};

use nova::{Diagnostic, KArgs, Scope, Severity, Value};

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

#[test]
fn order_workflow_threads_state_templates_and_diagnostics_together() {
    let receipt = Arc::new(Mutex::new(String::new()));
    let sink = receipt.clone();
    let runtime = nova::new()
        .var("store", "nova-mart")
        .template("receipt", "{{ store }}: {{ qty }} x {{ unit }} = {{ total }}")
        .predicate("in_stock", |_args: &[Value], kargs: &KArgs, _scope: &Scope| {
            Ok(kargs.get("qty") > Some(&Value::from(0)))
        })
        .func("subtotal", |_args: &[Value], kargs: &KArgs, scope: &Scope| -> FuncResult {
            let qty = kargs.get("qty").and_then(|v| u64::try_from(v.clone()).ok()).unwrap_or(0);
            let unit = kargs.get("unit").and_then(|v| u64::try_from(v.clone()).ok()).unwrap_or(0);
            nova::info!("priced {} units", qty).emit(scope);
            Ok(Some(Value::from(qty * unit)))
        })
        .action(
            "fulfill",
            move |args: &[Value], kargs: &KArgs, scope: &Scope| -> ActionResult {
                nova::warn!("fulfilling order").emit(scope);

                let total = nova::call!("subtotal", *args, **kargs as u64).unwrap_or(0);
                nova::set!("total", total);

                *sink.lock().unwrap() = scope.render("receipt")?;
                Ok(())
            },
        )
        .action("reject", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            nova::error!("out of stock").emit(scope);
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

    let output = runtime.call("process", [("qty", 3), ("unit", 5)]).unwrap();

    assert_eq!(*receipt.lock().unwrap(), "nova-mart: 3 x 5 = 15");

    let messages = collect_messages(&output.diagnostics);
    assert!(messages.contains(&"fulfilling order".to_string()), "{messages:?}");
    assert!(messages.contains(&"priced 3 units".to_string()), "{messages:?}");
    assert!(
        !messages.contains(&"out of stock".to_string()),
        "reject branch should not run"
    );
    assert_eq!(output.diagnostics[0].severity(), Severity::Warn);
}

#[test]
fn order_workflow_else_branch_rejects_and_escalates_severity() {
    let receipt = Arc::new(Mutex::new(String::from("untouched")));
    let sink = receipt.clone();
    let runtime = nova::new()
        .predicate("in_stock", |_args: &[Value], kargs: &KArgs, _scope: &Scope| {
            Ok(kargs.get("qty") > Some(&Value::from(0)))
        })
        .action(
            "fulfill",
            move |_args: &[Value], _kargs: &KArgs, _scope: &Scope| -> ActionResult {
                *sink.lock().unwrap() = String::from("fulfilled");
                Ok(())
            },
        )
        .action("reject", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            nova::error!("out of stock").emit(scope);
            assert!(!nova::has!("total"));
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

    let output = runtime.call("process", [("qty", 0)]).unwrap();

    assert_eq!(*receipt.lock().unwrap(), "untouched");
    let messages = collect_messages(&output.diagnostics);
    assert!(messages.contains(&"out of stock".to_string()), "{messages:?}");
    assert_eq!(output.diagnostics[0].severity(), Severity::Error);
}

#[test]
fn recursive_calls_accumulate_state_and_coerce_results() {
    let runtime = nova::new()
        .predicate("is_zero", |_args: &[Value], kargs: &KArgs, _scope: &Scope| {
            Ok(kargs.get("n") == Some(&Value::from(0)))
        })
        .predicate("is_positive", |args: &[Value], kargs: &KArgs, scope: &Scope| {
            Ok(!nova::call!("is_zero", *args, **kargs).map(|v| v.is_true()).unwrap_or(false))
        })
        .func("fact", |_args: &[Value], kargs: &KArgs, scope: &Scope| -> FuncResult {
            let n = kargs.get("n").and_then(|v| u64::try_from(v.clone()).ok()).unwrap_or(0);

            if n == 0 {
                return Ok(Some(Value::from(1u64)));
            }

            let sub = nova::call!("fact", n = n - 1 as u64).unwrap_or(1);
            Ok(Some(Value::from(n * sub)))
        })
        .action("run", |args: &[Value], kargs: &KArgs, scope: &Scope| -> ActionResult {
            assert!(nova::call!("is_positive", *args, **kargs).unwrap().is_true());

            let result = nova::call!("fact", *args, **kargs as u64).unwrap_or(0);
            nova::set!("result", result);
            nova::info!("factorial = {}", result).emit(scope);
            Ok(())
        })
        .build()
        .unwrap();

    let output = runtime.call("run", [("n", 5)]).unwrap();
    let messages = collect_messages(&output.diagnostics);
    assert!(messages.contains(&"factorial = 120".to_string()), "{messages:?}");

    let value = runtime.func("fact", [("n", 5)]).unwrap().value;
    assert_eq!(value, Some(Value::from(120u64)));
}

#[test]
fn template_composes_callables_and_captures_their_diagnostics() {
    let out = Arc::new(Mutex::new(String::new()));
    let sink = out.clone();
    let runtime = nova::new()
        .var("label", "score")
        .func("double", |args: &[Value], _kargs: &KArgs, scope: &Scope| -> FuncResult {
            let n = args.first().and_then(|v| u64::try_from(v.clone()).ok()).unwrap_or(0);
            nova::warn!("doubling {}", n).emit(scope);
            Ok(Some(Value::from(n * 2)))
        })
        .predicate("is_positive", |args: &[Value], _kargs: &KArgs, _scope: &Scope| {
            Ok(args.first() > Some(&Value::from(0)))
        })
        .action(
            "render",
            move |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
                *sink.lock().unwrap() = scope.render_str("{{ label }}: {{ double(21) }} ({{ is_positive(1) }})")?;
                Ok(())
            },
        )
        .build()
        .unwrap();

    let output = runtime.call("render", KArgs::new()).unwrap();

    assert_eq!(*out.lock().unwrap(), "score: 42 (true)");
    let messages = collect_messages(&output.diagnostics);
    assert!(messages.contains(&"doubling 21".to_string()), "{messages:?}");
}

#[test]
fn eval_predicate_and_call_isolation_across_a_chain() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let calls = seen.clone();
    let runtime = nova::new()
        .predicate("even", |_args: &[Value], kargs: &KArgs, _scope: &Scope| {
            let n = kargs.get("n").and_then(|v| u64::try_from(v.clone()).ok()).unwrap_or(1);
            Ok(n.is_multiple_of(2))
        })
        .action(
            "callee",
            move |_args: &[Value], kargs: &KArgs, _scope: &Scope| -> ActionResult {
                calls.lock().unwrap().push(kargs.clone());
                Ok(())
            },
        )
        .action("caller", |_args: &[Value], kargs: &KArgs, scope: &Scope| -> ActionResult {
            assert_eq!(kargs.get("n"), Some(&Value::from(1)));
            nova::call!("callee", n = 2);
            assert_eq!(kargs.get("n"), Some(&Value::from(1)));
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

#[test]
fn set_registers_bindings_into_the_live_scope() {
    let out = Arc::new(Mutex::new(String::new()));
    let sink = out.clone();
    let runtime = nova::new()
        .action(
            "setup",
            move |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
                nova::set!("greeting", "hello");
                nova::set!(
                    "shout",
                    nova::Object::func("shout", |_args: &[Value], kargs: &KArgs, _scope: &Scope| -> FuncResult {
                        let word = kargs.get("word").map(|v| v.to_string()).unwrap_or_default();
                        Ok(Some(Value::from(word.to_uppercase())))
                    })
                );

                let loud = nova::call!("shout", word = "hey").unwrap();
                assert_eq!(loud, Value::from("HEY"));

                *sink.lock().unwrap() = scope.render_str("{{ greeting }}")?;
                Ok(())
            },
        )
        .build()
        .unwrap();

    runtime.call("setup", KArgs::new()).unwrap();
    assert_eq!(*out.lock().unwrap(), "hello");
}

#[test]
fn error_paths_surface_at_call_and_build_time() {
    let runtime = nova::new()
        .func("boom", |_args: &[Value], _kargs: &KArgs, _scope: &Scope| -> FuncResult {
            Err(nova::Error::message("kaboom").into())
        })
        .action("caller", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            nova::call!("boom");
            Ok(())
        })
        .build()
        .unwrap();

    assert!(runtime.call("missing", KArgs::new()).is_err());
    assert!(runtime.call("caller", KArgs::new()).is_err());
    assert!(nova::new().template("t", "{{ ").build().is_err());
}

#[test]
fn manifest_hydrates_into_a_runnable_runtime() {
    let manifest = nova::manifest()
        .name("flow")
        .var("greeting", "hello")
        .var("count", 3)
        .template("banner", "== {{ greeting }} ==")
        .step(nova::step().name("say").run("{{ info(greeting ~ ' world') }}"))
        .step(nova::step().guard("count > 10").run("{{ info('skipped') }}"))
        .step(
            nova::step()
                .guard("count > 0")
                .run("{% if count > 0 %}{{ info('count is positive') }}{% endif %}"),
        )
        .step(nova::step().call("missing_func", [("k", "v")]))
        .build();

    let runtime = nova::Runtime::try_from(manifest).unwrap();
    let output = runtime.call("flow", KArgs::new()).unwrap();
    let messages = collect_messages(&output.diagnostics);

    assert!(messages.iter().any(|m| m == "hello world"), "{messages:?}");
    assert!(!messages.iter().any(|m| m == "skipped"), "{messages:?}");
    assert!(messages.iter().any(|m| m == "count is positive"), "{messages:?}");
    assert!(messages.iter().any(|m| m.contains("missing_func")), "{messages:?}");
}

#[test]
fn builtin_diagnostic_functions_emit_positionally_from_a_block() {
    let runtime = nova::manifest()
        .name("flow")
        .var("items", vec!["a", "b", "c"])
        .step(nova::step().run("{% for item in items %}{{ info(item) }}{% endfor %}"))
        .step(nova::step().run("{{ warn('careful') }}"))
        .step(nova::step().run("{{ error('boom') }}"))
        .step(nova::step().run("{{ print('to stdout') }}{{ println('and a line') }}"))
        .build();

    let runtime = nova::Runtime::try_from(runtime).unwrap();
    let output = runtime.call("flow", KArgs::new()).unwrap();
    let messages = collect_messages(&output.diagnostics);

    for item in ["a", "b", "c"] {
        assert!(messages.iter().any(|m| m == item), "{messages:?}");
    }
    assert!(messages.iter().any(|m| m == "careful"), "{messages:?}");
    assert!(messages.iter().any(|m| m == "boom"), "{messages:?}");
    assert!(!messages.iter().any(|m| m == "to stdout"), "{messages:?}");
    assert!(!messages.iter().any(|m| m == "and a line"), "{messages:?}");

    assert_eq!(output.diagnostics[0].severity(), Severity::Error);
}

#[test]
fn shell_step_runs_commands_and_reports_failures() {
    let runtime = nova::manifest()
        .name("flow")
        .var("greeting", "hello")
        .step(nova::step().shell("echo {{ greeting }}"))
        .step(nova::step().shell("exit 3"))
        .step(nova::step().run("{{ info('after failure') }}"))
        .build();

    let runtime = nova::Runtime::try_from(runtime).unwrap();
    let output = runtime.call("flow", KArgs::new()).unwrap();
    let messages = collect_messages(&output.diagnostics);

    assert!(!messages.iter().any(|m| m == "hello"), "{messages:?}");
    assert!(messages.iter().any(|m| m.contains("shell exited")), "{messages:?}");
    assert!(messages.iter().any(|m| m == "after failure"), "{messages:?}");
    assert_eq!(output.diagnostics[0].severity(), Severity::Error);
}

#[test]
fn positional_required_with_optional_keyword_arg() {
    let out = Arc::new(Mutex::new(String::new()));
    let sink = out.clone();
    let runtime = nova::new()
        .func("greet", |args: &[Value], kargs: &KArgs, _scope: &Scope| -> FuncResult {
            let name = args.first().map(|v| v.to_string()).unwrap_or_default();
            let suffix = kargs.get("suffix").map(|v| v.to_string()).unwrap_or_default();
            Ok(Some(Value::from(format!("{name}{suffix}"))))
        })
        .action(
            "render",
            move |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
                *sink.lock().unwrap() = scope.render_str("{{ greet('bob', suffix='!') }}")?;
                Ok(())
            },
        )
        .build()
        .unwrap();

    runtime.call("render", KArgs::new()).unwrap();
    assert_eq!(*out.lock().unwrap(), "bob!");
}

#[test]
fn namespaced_manifests_resolve_across_scopes() {
    let main = nova::manifest()
        .name("main")
        .on([nova::Trigger::Run { priority: Some(5) }])
        .var("count", 3)
        .step(nova::step().name("greet").run("{{ info(lib.greeting) }}"))
        .build();
    let lib = nova::manifest()
        .name("lib")
        .on([nova::Trigger::Call])
        .var("greeting", "hello")
        .build();
    let runtime = nova::Runtime::try_from(vec![main, lib]).unwrap();
    let out = runtime.call("main", KArgs::new()).unwrap();
    let messages = collect_messages(&out.diagnostics);

    assert!(messages.iter().any(|m| m == "hello"), "{messages:?}");
    assert!(
        out.diagnostics.iter().all(|d| d.severity() != Severity::Error),
        "cross-namespace resolution should not error: {messages:?}"
    );
}
