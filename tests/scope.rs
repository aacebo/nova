use nova::{KArgs, Scope, Value, Var, del, get, get_mut, has, set};

type ActionResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn scope_bindings_lifecycle_through_macros() {
    let runtime = nova::new()
        .action("run", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            let baseline = scope.len();
            assert!(!has!("x"));

            set!("x", Var::new("x", 1));
            set!("x", Var::new("x", 2));
            assert!(has!("x"));
            assert_eq!(scope.len(), baseline + 1);
            assert_eq!(get!("x").unwrap().as_var().unwrap().value, Value::from(2));

            {
                let mut slot = get_mut!("x").expect("x should be set");
                slot.as_var_mut().unwrap().value = Value::from(9);
            }
            assert_eq!(get!("x").unwrap().as_var().unwrap().value, Value::from(9));

            del!("x");
            assert!(!has!("x"));
            assert!(get!("x").is_none());
            assert_eq!(scope.len(), baseline);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("run", KArgs::new()).unwrap();
}

#[test]
fn forked_scopes_resolve_and_write_through_to_ancestors() {
    let runtime = nova::new()
        .action("child", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            assert_eq!(get!("base").unwrap().as_var().unwrap().value, Value::from(1));

            set!("base", Var::new("base", 2));
            set!("fresh", Var::new("fresh", 7));
            Ok(())
        })
        .action("parent", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            set!("base", Var::new("base", 1));

            nova::call!("child");

            assert_eq!(get!("base").unwrap().as_var().unwrap().value, Value::from(2));
            assert_eq!(get!("fresh").unwrap().as_var().unwrap().value, Value::from(7));
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("parent", KArgs::new()).unwrap();
}

#[test]
fn child_deletion_recurses_to_the_owning_ancestor() {
    let runtime = nova::new()
        .action("child", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            assert!(has!("x"));
            del!("x");
            Ok(())
        })
        .action("parent", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            set!("x", Var::new("x", 1));

            nova::call!("child");

            assert!(!has!("x"));
            assert!(get!("x").is_none());
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("parent", KArgs::new()).unwrap();
}

#[test]
fn typed_get_filters_by_object_variant() {
    let runtime = nova::new()
        .action("run", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            set!("x", Var::new("x", 1));

            {
                let mut slot = get_mut!("x" as Var).expect("x is a Var");
                slot.as_var_mut().unwrap().value = Value::from(9);
            }

            assert_eq!(get!("x" as Var).unwrap().as_var().unwrap().value, Value::from(9));
            assert!(get!("x" as Function).is_none());
            assert!(get_mut!("x" as Function).is_none());
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("run", KArgs::new()).unwrap();
}
