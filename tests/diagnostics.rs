use nova::{KArgs, Severity, Value, error, info, scope, warn};

type ActionResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn nested_diagnostics_thread_trace_id_and_roll_up_severity() {
    let runtime = nova::new()
        .action("run", |_args: &[Value], _kargs: &KArgs| -> ActionResult {
            let trace_id = *scope().trace_id();

            let d = info!("request {}", 7 ; [
                info!("validated"),
                warn!("slow path"),
                error!("code {}", 500),
            ]);

            assert_eq!(d.trace_id, trace_id);
            assert_eq!(d.severity, Some(Severity::Info));
            assert_eq!(d.children.len(), 3);
            assert_eq!(d.children[2].message.as_deref(), Some("code 500"));
            assert!(d.children.iter().all(|c| c.trace_id == trace_id));
            assert_eq!(d.severity(), Severity::Error);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("run", KArgs::new()).unwrap();
}

#[test]
fn fluent_builders_and_emit_populate_the_scope_buffer() {
    let runtime = nova::new()
        .action("run", |_args: &[Value], _kargs: &KArgs| -> ActionResult {
            warn!("job").warn("step a").error("step b").emit();
            info!("done").emit();
            Ok(())
        })
        .build()
        .unwrap();

    let diagnostics = runtime.call("run", KArgs::new()).unwrap().diagnostics;
    assert_eq!(diagnostics.len(), 1);

    let emitted = &diagnostics[0].children;
    assert_eq!(emitted.len(), 2);

    let job = &emitted[0];
    assert_eq!(job.message.as_deref(), Some("job"));
    assert_eq!(job.children.len(), 2);
    assert_eq!(job.children[0].message.as_deref(), Some("step a"));
    assert_eq!(job.severity(), Severity::Error);

    assert_eq!(emitted[1].message.as_deref(), Some("done"));
}

#[test]
fn diagnostics_built_outside_an_invocation_get_fresh_ids() {
    let a = warn!("orphan");
    let b = warn!("orphan");
    assert_ne!(a.trace_id, b.trace_id);
}
