use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use nova::{KArgs, Scope, Severity, Value, del, get, get_mut, has, set};

type ActionResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn scope_bindings_lifecycle_through_macros() {
    let ran = Arc::new(AtomicBool::new(false));
    let flag = ran.clone();

    let runtime = nova::new()
        .action("run", move |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            let baseline = scope.len();
            assert!(!has!("x"));

            set!("x", 1);
            set!("x", 2);
            assert!(has!("x"));
            assert_eq!(scope.len(), baseline + 1);
            assert_eq!(get!("x").unwrap().clone(), Value::from(2));

            {
                let mut slot = get_mut!("x").expect("x should be set");
                *slot.as_value_mut().unwrap() = Value::from(9);
            }
            assert_eq!(get!("x").unwrap().clone(), Value::from(9));

            del!("x");
            assert!(!has!("x"));
            assert!(get!("x").is_none());
            assert_eq!(scope.len(), baseline);

            flag.store(true, Ordering::SeqCst);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("run", KArgs::new()).unwrap();
    assert!(ran.load(Ordering::SeqCst), "run action never executed");
}

#[test]
fn forked_scopes_resolve_and_write_through_to_ancestors() {
    let ran = Arc::new(AtomicBool::new(false));
    let flag = ran.clone();

    let runtime = nova::new()
        .action("child", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            assert_eq!(get!("base").unwrap().clone(), Value::from(1));

            set!("base", 2);
            set!("fresh", 7);
            Ok(())
        })
        .action(
            "parent",
            move |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
                set!("base", 1);

                nova::call!("child");

                assert_eq!(get!("base").unwrap().clone(), Value::from(2));
                assert_eq!(get!("fresh").unwrap().clone(), Value::from(7));

                flag.store(true, Ordering::SeqCst);
                Ok(())
            },
        )
        .build()
        .unwrap();

    let output = runtime.call("parent", KArgs::new()).unwrap();
    assert!(ran.load(Ordering::SeqCst), "parent action never executed");
    assert!(
        output.diagnostics.iter().all(|d| d.severity() != Severity::Error),
        "no child call should have errored: {:?}",
        output.diagnostics
    );
}

#[test]
fn child_deletion_recurses_to_the_owning_ancestor() {
    let ran = Arc::new(AtomicBool::new(false));
    let flag = ran.clone();

    let runtime = nova::new()
        .action("child", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            assert!(has!("x"));
            del!("x");
            Ok(())
        })
        .action(
            "parent",
            move |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
                set!("x", 1);

                nova::call!("child");

                assert!(!has!("x"));
                assert!(get!("x").is_none());

                flag.store(true, Ordering::SeqCst);
                Ok(())
            },
        )
        .build()
        .unwrap();

    let output = runtime.call("parent", KArgs::new()).unwrap();
    assert!(ran.load(Ordering::SeqCst), "parent action never executed");
    assert!(
        output.diagnostics.iter().all(|d| d.severity() != Severity::Error),
        "no child call should have errored: {:?}",
        output.diagnostics
    );
}

#[test]
fn typed_get_filters_by_object_variant() {
    let ran = Arc::new(AtomicBool::new(false));
    let flag = ran.clone();

    let runtime = nova::new()
        .action("run", move |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            set!("x", 1);

            {
                let mut slot = get_mut!("x" as Value).expect("x is a Var");
                *slot.as_value_mut().unwrap() = Value::from(9);
            }

            assert_eq!(get!("x" as Value).unwrap().clone(), Value::from(9));
            assert!(get!("x" as Function).is_none());
            assert!(get_mut!("x" as Function).is_none());

            flag.store(true, Ordering::SeqCst);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("run", KArgs::new()).unwrap();
    assert!(ran.load(Ordering::SeqCst), "run action never executed");
}
