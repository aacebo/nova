use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use nova::{KArgs, Scope, Value, args, get, set};

type ActionResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn scope_moves_across_threads_without_panicking() {
    let ran = Arc::new(AtomicBool::new(false));
    let flag = ran.clone();
    let runtime = nova::new()
        .action("run", move |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            set!("base", 1);

            let child = scope.fork("worker", Vec::new(), KArgs::new());
            let handle = std::thread::spawn(move || {
                let scope = &child;
                set!("threaded", 42);
                get!("base").unwrap().as_value().unwrap().clone()
            });

            let seen = handle.join().unwrap();
            assert_eq!(seen, Value::from(1));

            flag.store(true, Ordering::SeqCst);
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("run", args!()).unwrap();
    assert!(ran.load(Ordering::SeqCst), "run action never executed");
}
