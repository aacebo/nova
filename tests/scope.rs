use nova::{Args, Scope, Value, Var, del, enter, get, get_mut, has, set};

/// Exercises the full single-scope lifecycle through the macros: create, overwrite,
/// mutate in place, read back, and delete — asserting `len`/`has` stay consistent.
#[test]
fn scope_bindings_lifecycle_through_macros() {
    let scope = Scope::new();
    let _guard = enter(&scope);

    assert!(!has!("x"));

    set!("x", Var::new("x", 1));
    set!("x", Var::new("x", 2)); // overwrite in place, not a second slot
    assert!(has!("x"));
    assert_eq!(scope.len(), 1);
    assert_eq!(get!("x").unwrap().as_var().unwrap().value, Value::from(2));

    {
        let mut slot = get_mut!("x").expect("x should be set");
        slot.as_var_mut().unwrap().value = Value::from(9);
    }
    assert_eq!(get!("x").unwrap().as_var().unwrap().value, Value::from(9));

    del!("x");
    assert!(!has!("x"));
    assert!(get!("x").is_none());
    assert_eq!(scope.len(), 0);
}

/// Fork/parent resolution and write-through semantics in one scenario: a child resolves
/// a parent binding, writes to an existing parent key (updating the parent), and a brand
/// new key lands at the root so every level can see it.
#[test]
fn forked_scopes_resolve_and_write_through_to_ancestors() {
    let root = Scope::new();
    root.set("base", Var::new("base", 1));

    let child = root.fork(Args::new());
    let grandchild = child.fork(Args::new());

    {
        let _guard = enter(&grandchild);

        // resolves the ancestor's binding
        assert_eq!(get!("base").unwrap().as_var().unwrap().value, Value::from(1));

        // writing an existing key updates the owning ancestor (root), not a local shadow
        set!("base", Var::new("base", 2));

        // a new key with no existing owner lands at the root
        set!("fresh", Var::new("fresh", 7));
    }

    assert_eq!(root.get("base").unwrap().as_var().unwrap().value, Value::from(2));
    assert_eq!(root.get("fresh").unwrap().as_var().unwrap().value, Value::from(7));
    // and the whole chain still resolves both
    assert!(grandchild.has("fresh"));
}

/// A child deletion recurses to the ancestor that owns the binding, removing it for the
/// whole chain.
#[test]
fn child_deletion_recurses_to_the_owning_ancestor() {
    let parent = Scope::new();
    parent.set("x", Var::new("x", 1));
    let child = parent.fork(Args::new());

    {
        let _guard = enter(&child);
        assert!(has!("x"));
        del!("x");
    }

    assert!(!parent.has("x"));
    assert!(parent.get("x").is_none());
}
