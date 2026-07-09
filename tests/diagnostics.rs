mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use common::Recorder;
use nova::{KArgs, Scope, Severity, Value, error, info, warn};

type ActionResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn nested_diagnostics_thread_trace_id_and_roll_up_severity() {
    let ran = Arc::new(AtomicBool::new(false));
    let flag = ran.clone();
    let runtime = nova::new()
        .action("run", move |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            let trace_id = *scope.trace_id();

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

            flag.store(true, Ordering::SeqCst);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("run", KArgs::new()).unwrap();
    assert!(ran.load(Ordering::SeqCst), "run action never executed");
}

#[test]
fn fluent_builders_and_emit_stream_each_diagnostic() {
    let recorder = Recorder::new();
    let runtime = nova::new()
        .observe(recorder.clone())
        .action("run", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            warn!("job").warn("step a").error("step b").emit(scope);
            info!("done").emit(scope);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("run", KArgs::new()).unwrap();
    drop(runtime);

    let emitted = recorder.diagnostics();
    assert_eq!(emitted.len(), 2);

    let job = &emitted[0];
    assert_eq!(job.message.as_deref(), Some("job"));
    assert_eq!(job.children.len(), 2);
    assert_eq!(job.children[0].message.as_deref(), Some("step a"));
    assert_eq!(job.severity(), Severity::Error);

    assert_eq!(emitted[1].message.as_deref(), Some("done"));
}
