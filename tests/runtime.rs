mod common;

use std::sync::{Arc, Mutex};

use common::Recorder;
use nova::reflect::Value;
use nova::template::{Args, Pointer};
use nova::{Manifest, Scope, args};

type ActionResult = Result<(), Box<dyn std::error::Error>>;
type FuncResult = Result<Pointer, Box<dyn std::error::Error>>;

fn routines(recorder: &Recorder, manifests: impl IntoIterator<Item = Manifest>) -> nova::Runtime {
    let mut builder = nova::new().observe(recorder.clone());

    for manifest in manifests {
        builder = builder.routine(manifest);
    }

    builder.build().unwrap()
}

#[test]
fn order_workflow_threads_state_templates_and_diagnostics_together() {
    let receipt = Arc::new(Mutex::new(String::new()));
    let sink = receipt.clone();
    let recorder = Recorder::new();
    let runtime = nova::new()
        .observe(recorder.clone())
        .var("store", "nova-mart")
        .template("receipt", "{{ store }}: {{ qty }} x {{ unit }} = {{ total }}")
        .predicate("in_stock", |args: &Args, _scope: &Scope| Ok(args.key("qty") > Value::from(0)))
        .func("subtotal", |args: &Args, scope: &Scope| -> FuncResult {
            let qty = u64::try_from(args.key("qty")).unwrap_or(0);
            let unit = u64::try_from(args.key("unit")).unwrap_or(0);
            nova::info!("priced {} units", qty).emit(scope);
            Ok(Value::from(qty * unit).into())
        })
        .action("fulfill", move |args: &Args, scope: &Scope| -> ActionResult {
            nova::warn!("fulfilling order").emit(scope);

            let total = nova::call!("subtotal", **args as u64);
            nova::set!("total", total);

            *sink.lock().unwrap() = scope.render("receipt")?;
            Ok(())
        })
        .action("reject", |_args: &Args, scope: &Scope| -> ActionResult {
            nova::error!("out of stock").emit(scope);
            Ok(())
        })
        .action("process", |args: &Args, scope: &Scope| -> ActionResult {
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
    drop(runtime);

    assert_eq!(*receipt.lock().unwrap(), "nova-mart: 3 x 5 = 15");

    let messages = recorder.messages();
    assert!(messages.contains(&"fulfilling order".to_string()), "{messages:?}");
    assert!(messages.contains(&"priced 3 units".to_string()), "{messages:?}");
    assert!(
        !messages.contains(&"out of stock".to_string()),
        "reject branch should not run"
    );
    assert!(!recorder.has_error());
}

#[test]
fn order_workflow_else_branch_rejects_and_escalates_severity() {
    let receipt = Arc::new(Mutex::new(String::from("untouched")));
    let sink = receipt.clone();
    let recorder = Recorder::new();
    let runtime = nova::new()
        .observe(recorder.clone())
        .predicate("in_stock", |args: &Args, _scope: &Scope| Ok(args.key("qty") > Value::from(0)))
        .action("fulfill", move |_args: &Args, _scope: &Scope| -> ActionResult {
            *sink.lock().unwrap() = String::from("fulfilled");
            Ok(())
        })
        .action("reject", |_args: &Args, scope: &Scope| -> ActionResult {
            nova::error!("out of stock").emit(scope);
            assert!(!nova::has!("total"));
            Ok(())
        })
        .action("process", |args: &Args, scope: &Scope| -> ActionResult {
            if nova::call!("in_stock", **args).is_truthy() {
                nova::call!("fulfill", **args);
            } else {
                nova::call!("reject", **args);
            }
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("process", args!(qty = 0)).unwrap();
    drop(runtime);

    assert_eq!(*receipt.lock().unwrap(), "untouched");
    assert!(
        recorder.messages().contains(&"out of stock".to_string()),
        "{:?}",
        recorder.messages()
    );
    assert!(recorder.has_error());
}

#[test]
fn recursive_calls_accumulate_state_and_coerce_results() {
    let recorder = Recorder::new();
    let runtime = nova::new()
        .observe(recorder.clone())
        .predicate("is_zero", |args: &Args, _scope: &Scope| Ok(args.key("n") == Value::from(0)))
        .predicate("is_positive", |args: &Args, scope: &Scope| {
            Ok(!nova::call!("is_zero", **args).is_truthy())
        })
        .func("fact", |args: &Args, scope: &Scope| -> FuncResult {
            let n = u64::try_from(args.key("n")).unwrap_or(0);

            if n == 0 {
                return Ok(Value::from(1u64).into());
            }

            let sub = nova::call!("fact", n = n - 1 as u64);
            Ok(Value::from(n * sub).into())
        })
        .action("run", |args: &Args, scope: &Scope| -> ActionResult {
            assert!(nova::call!("is_positive", **args).is_truthy());

            let result = nova::call!("fact", **args as u64);
            nova::set!("result", result);
            nova::info!("factorial = {}", result).emit(scope);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("run", args!(n = 5)).unwrap();
    let value = runtime.func("fact", args!(n = 5)).unwrap();
    assert_eq!(value, Value::from(120u64));

    drop(runtime);
    assert!(
        recorder.messages().contains(&"factorial = 120".to_string()),
        "{:?}",
        recorder.messages()
    );
}

#[test]
fn template_composes_callables_and_captures_their_diagnostics() {
    let out = Arc::new(Mutex::new(String::new()));
    let sink = out.clone();
    let recorder = Recorder::new();
    let runtime = nova::new()
        .observe(recorder.clone())
        .var("label", "score")
        .func("double", |args: &Args, scope: &Scope| -> FuncResult {
            let n = u64::try_from(args.at(0)).unwrap_or(0);
            nova::warn!("doubling {}", n).emit(scope);
            Ok(Value::from(n * 2).into())
        })
        .predicate("is_positive", |args: &Args, _scope: &Scope| Ok(args.at(0) > Value::from(0)))
        .action("render", move |_args: &Args, scope: &Scope| -> ActionResult {
            *sink.lock().unwrap() = scope.render_str("{{ label }}: {{ double(21) }} ({{ is_positive(1) }})")?;
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("render", args!()).unwrap();
    drop(runtime);

    assert_eq!(*out.lock().unwrap(), "score: 42 (true)");
    assert!(
        recorder.messages().contains(&"doubling 21".to_string()),
        "{:?}",
        recorder.messages()
    );
}

#[test]
fn eval_predicate_and_call_isolation_across_a_chain() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let calls = seen.clone();
    let runtime = nova::new()
        .predicate("even", |args: &Args, _scope: &Scope| {
            let n = u64::try_from(args.key("n")).unwrap_or(1);
            Ok(n.is_multiple_of(2))
        })
        .action("callee", move |args: &Args, _scope: &Scope| -> ActionResult {
            calls.lock().unwrap().push(args.kargs().clone());
            Ok(())
        })
        .action("caller", |args: &Args, scope: &Scope| -> ActionResult {
            assert_eq!(args.key("n"), Value::from(1));
            nova::call!("callee", n = 2);
            assert_eq!(args.key("n"), Value::from(1));
            Ok(())
        })
        .build()
        .unwrap();

    assert!(runtime.eval("even", args!(n = 4)).unwrap());
    assert!(!runtime.eval("even", args!(n = 3)).unwrap());

    runtime.call("caller", args!(n = 1)).unwrap();
    let recorded = seen.lock().unwrap();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].get("n").map(|v| v.value()), Some(Value::from(2)));
}

#[test]
fn set_registers_bindings_into_the_live_scope() {
    let out = Arc::new(Mutex::new(String::new()));
    let sink = out.clone();
    let runtime = nova::new()
        .action("setup", move |_args: &Args, scope: &Scope| -> ActionResult {
            nova::set!("greeting", "hello");
            nova::set!(
                "shout",
                nova::Binding::func("shout", |args: &Args, _scope: &Scope| -> FuncResult {
                    let word = args.key("word").to_string();
                    Ok(Value::from(word.to_uppercase()).into())
                })
            );

            let loud = nova::call!("shout", word = "hey");
            assert_eq!(loud, Value::from("HEY".to_string()));

            *sink.lock().unwrap() = scope.render_str("{{ greeting }}")?;
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("setup", args!()).unwrap();
    assert_eq!(*out.lock().unwrap(), "hello");
}

#[test]
fn error_paths_surface_at_call_and_build_time() {
    let runtime = nova::new()
        .func("boom", |_args: &Args, _scope: &Scope| -> FuncResult {
            Err(nova::Error::message("kaboom").into())
        })
        .action("caller", |_args: &Args, scope: &Scope| -> ActionResult {
            nova::call!("boom");
            Ok(())
        })
        .build()
        .unwrap();

    assert!(runtime.call("missing", args!()).is_err());
    assert!(runtime.call("caller", args!()).is_err());
    assert!(nova::new().template("t", "{{ ").build().is_err());
}

#[test]
fn manifest_hydrates_into_a_runnable_runtime() {
    let recorder = Recorder::new();
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
        .step(nova::step().call("missing_func", [] as [Value; 0], [("k", "v")]))
        .build();

    let runtime = routines(&recorder, [manifest]);
    runtime.call("flow", args!()).unwrap();
    drop(runtime);

    let messages = recorder.messages();
    assert!(messages.iter().any(|m| m == "hello world"), "{messages:?}");
    assert!(!messages.iter().any(|m| m == "skipped"), "{messages:?}");
    assert!(messages.iter().any(|m| m == "count is positive"), "{messages:?}");
    assert!(messages.iter().any(|m| m.contains("missing_func")), "{messages:?}");
}

#[test]
fn call_step_passes_positional_and_named_args() {
    let seen = Arc::new(Mutex::new(Vec::<(Pointer, Pointer)>::new()));
    let sink = seen.clone();
    let recorder = Recorder::new();

    let manifest = nova::manifest()
        .name("flow")
        .var("greeting", "hello")
        .step(nova::step().call("record", ["greeting"], [("tag", "'literal'")]))
        .build();

    let mut builder = nova::new().observe(recorder.clone());
    builder = builder.action("record", move |args: &Args, _scope: &Scope| -> ActionResult {
        sink.lock().unwrap().push((args.at(0), args.key("tag")));
        Ok(())
    });
    let runtime = builder.routine(manifest).build().unwrap();

    runtime.call("flow", args!()).unwrap();
    drop(runtime);

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 1, "record should have run once");
    assert_eq!(
        seen[0].0.value().as_str().map(|s| s.to_string()),
        Some("hello".to_string()),
        "positional arg resolves via var"
    );
    assert_eq!(
        seen[0].1.value().as_str().map(|s| s.to_string()),
        Some("literal".to_string()),
        "named arg resolves as expression"
    );
    assert!(!recorder.has_error(), "call step should not error");
}

#[test]
fn args_macro_routes_positional_kwargs_and_empty() {
    let runtime = nova::new()
        .func("echo", |args: &Args, _scope: &Scope| -> FuncResult {
            let pos = args.at(0).value().to_string();
            let tag = args.key("tag").value().to_string();
            Ok(Value::from(format!("{pos}|{tag}")).into())
        })
        .build()
        .unwrap();

    let undef = "<undefined>";

    // empty
    let out = runtime.func("echo", args!()).unwrap();
    assert_eq!(out.value().to_string(), format!("{undef}|{undef}"));

    // positional only
    let out = runtime.func("echo", args!("hi")).unwrap();
    assert_eq!(out.value().to_string(), format!("hi|{undef}"));

    // kwargs only
    let out = runtime.func("echo", args!(tag = "t")).unwrap();
    assert_eq!(out.value().to_string(), format!("{undef}|t"));

    // both
    let out = runtime.func("echo", args!("hi", tag = "t")).unwrap();
    assert_eq!(out.value().to_string(), "hi|t");
}

#[test]
fn builtin_diagnostic_functions_emit_positionally_from_a_block() {
    let recorder = Recorder::new();
    let manifest = nova::manifest()
        .name("flow")
        .var("items", vec!["a", "b", "c"])
        .step(nova::step().run("{% for item in items %}{{ info(item) }}{% endfor %}"))
        .step(nova::step().run("{{ warn('careful') }}"))
        .step(nova::step().run("{{ error('boom') }}"))
        .step(nova::step().run("{{ print('to stdout') }}{{ println('and a line') }}"))
        .build();

    let runtime = routines(&recorder, [manifest]);
    runtime.call("flow", args!()).unwrap();
    drop(runtime);

    let messages = recorder.messages();
    for item in ["a", "b", "c"] {
        assert!(messages.iter().any(|m| m == item), "{messages:?}");
    }
    assert!(messages.iter().any(|m| m == "careful"), "{messages:?}");
    assert!(messages.iter().any(|m| m == "boom"), "{messages:?}");
    assert!(!messages.iter().any(|m| m == "to stdout"), "{messages:?}");
    assert!(!messages.iter().any(|m| m == "and a line"), "{messages:?}");
    assert!(recorder.has_error());
}

#[test]
fn shell_step_runs_commands_and_reports_failures() {
    let recorder = Recorder::new();
    let manifest = nova::manifest()
        .name("flow")
        .var("greeting", "hello")
        .step(nova::step().shell("echo {{ greeting }}"))
        .step(nova::step().shell("exit 3"))
        .step(nova::step().run("{{ info('after failure') }}"))
        .build();

    let runtime = routines(&recorder, [manifest]);
    runtime.call("flow", args!()).unwrap();
    drop(runtime);

    let messages = recorder.messages();
    assert!(!messages.iter().any(|m| m == "hello"), "{messages:?}");
    assert!(messages.iter().any(|m| m.contains("shell exited")), "{messages:?}");
    assert!(messages.iter().any(|m| m == "after failure"), "{messages:?}");
    assert!(recorder.has_error());
}

#[test]
fn positional_required_with_optional_keyword_arg() {
    let out = Arc::new(Mutex::new(String::new()));
    let sink = out.clone();
    let runtime = nova::new()
        .func("greet", |args: &Args, _scope: &Scope| -> FuncResult {
            let name = args.at(0).to_string();
            let suffix = args.key("suffix").to_string();
            Ok(Value::from(format!("{name}{suffix}")).into())
        })
        .action("render", move |_args: &Args, scope: &Scope| -> ActionResult {
            *sink.lock().unwrap() = scope.render_str("{{ greet('bob', suffix='!') }}")?;
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("render", args!()).unwrap();
    assert_eq!(*out.lock().unwrap(), "bob!");
}

#[test]
fn namespaced_manifests_resolve_across_scopes() {
    let recorder = Recorder::new();
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

    let runtime = routines(&recorder, [main, lib]);
    runtime.call("main", args!()).unwrap();
    drop(runtime);

    let messages = recorder.messages();
    assert!(messages.iter().any(|m| m == "hello"), "{messages:?}");
    assert!(
        !recorder.has_error(),
        "cross-namespace resolution should not error: {messages:?}"
    );
}

#[test]
fn runtime_render_resolves_named_template_against_root_vars() {
    let runtime = nova::new()
        .var("store", "nova-mart")
        .var("qty", 3)
        .template("receipt", "{{ store }}: {{ qty }}")
        .build()
        .unwrap();

    assert_eq!(runtime.render("receipt").unwrap(), "nova-mart: 3");
}

#[test]
fn runtime_render_str_renders_inline_source_against_root_vars() {
    let runtime = nova::new().var("greeting", "hello").build().unwrap();

    assert_eq!(runtime.render_str("{{ greeting }}!").unwrap(), "hello!");
}

#[test]
fn runtime_render_errors_on_unknown_template() {
    let runtime = nova::new().build().unwrap();
    assert!(runtime.render("missing").is_err());
}

#[test]
fn runtime_render_str_errors_on_invalid_source() {
    let runtime = nova::new().build().unwrap();
    assert!(runtime.render_str("{{ ").is_err());
}
