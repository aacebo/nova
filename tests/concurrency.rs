use nova::{KArgs, Scope, Value, Var, get, set};

type ActionResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn scope_moves_across_threads_without_panicking() {
    let runtime = nova::new()
        .action("run", |_args: &[Value], _kargs: &KArgs, scope: &Scope| -> ActionResult {
            set!("base", Var::new("base", 1));

            let child = scope.fork("worker", Vec::new(), KArgs::new());
            let handle = std::thread::spawn(move || {
                let scope = &child;
                set!("threaded", Var::new("threaded", 42));
                get!("base").unwrap().as_var().unwrap().value.clone()
            });

            let seen = handle.join().unwrap();
            assert_eq!(seen, Value::from(1));
            Ok(())
        })
        .build()
        .unwrap();

    runtime.call("run", KArgs::new()).unwrap();
}
