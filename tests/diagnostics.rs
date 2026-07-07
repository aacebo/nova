use nova::{Scope, Severity, enter, error, info, warn};

/// Builds a nested diagnostic tree with format-args messages and asserts the ambient
/// trace_id is threaded through every node, the child list is preserved, and severity
/// rolls up to the max across the tree.
#[test]
fn nested_diagnostics_thread_trace_id_and_roll_up_severity() {
    let scope = Scope::new();
    let _guard = enter(&scope);

    let d = info!("request {}", 7 ; [
        info!("validated"),
        warn!("slow path"),
        error!("code {}", 500),
    ]);

    assert_eq!(d.trace_id, *scope.trace_id());
    assert_eq!(d.severity, Some(Severity::Info));
    assert_eq!(d.children.len(), 3);
    assert_eq!(d.children[2].message.as_deref(), Some("code 500"));
    // every child inherited the ambient trace_id
    assert!(d.children.iter().all(|c| c.trace_id == *scope.trace_id()));
    // own severity is Info but an Error child escalates the reported severity
    assert_eq!(d.severity(), Severity::Error);
}

/// The fluent child builders on `Diagnostic` and `.emit()` in one flow: build a parent
/// with appended children, emit it plus a standalone diagnostic, then drain the scope
/// and confirm both landed with the right shape.
#[test]
fn fluent_builders_and_emit_populate_the_scope_buffer() {
    let scope = Scope::new();
    let _guard = enter(&scope);

    warn!("job").warn("step a").error("step b").emit();
    info!("done").emit();

    let diagnostics = scope.take_diagnostics();
    assert_eq!(diagnostics.len(), 2);

    let job = &diagnostics[0];
    assert_eq!(job.message.as_deref(), Some("job"));
    assert_eq!(job.children.len(), 2);
    assert_eq!(job.children[0].message.as_deref(), Some("step a"));
    assert_eq!(job.severity(), Severity::Error); // rolled up from `step b`

    assert_eq!(diagnostics[1].message.as_deref(), Some("done"));

    // draining emptied the buffer
    assert!(scope.take_diagnostics().is_empty());
}

/// Outside any invocation the macros still build (rather than panic), minting a fresh
/// trace_id per call so orphan diagnostics remain uniquely identifiable.
#[test]
fn diagnostics_built_outside_an_invocation_get_fresh_ids() {
    let a = warn!("orphan");
    let b = warn!("orphan");
    assert_ne!(a.trace_id, b.trace_id);
}
