use crate::{Builder, Diagnostic, KArgs, Scope, Severity, Value};

pub fn register(builder: Builder) -> Builder {
    builder
        .func("info", |args: &[Value], kargs: &KArgs, scope: &Scope| {
            emit(Severity::Info, args, kargs, scope);
            Ok(Value::from(()))
        })
        .func("warn", |args: &[Value], kargs: &KArgs, scope: &Scope| {
            emit(Severity::Warn, args, kargs, scope);
            Ok(Value::from(()))
        })
        .func("error", |args: &[Value], kargs: &KArgs, scope: &Scope| {
            emit(Severity::Error, args, kargs, scope);
            Ok(Value::from(()))
        })
        .func("print", |args: &[Value], kargs: &KArgs, _scope: &Scope| {
            print!("{}", message(args, kargs));
            Ok(Value::from(()))
        })
        .func("println", |args: &[Value], kargs: &KArgs, _scope: &Scope| {
            println!("{}", message(args, kargs));
            Ok(Value::from(()))
        })
}

fn message(args: &[Value], kargs: &KArgs) -> String {
    args.first()
        .or_else(|| kargs.get("message"))
        .map(|v| v.to_string())
        .unwrap_or_default()
}

fn emit(severity: Severity, args: &[Value], kargs: &KArgs, scope: &Scope) {
    scope.emit(Diagnostic::new(*scope.trace_id()).sev(severity).message(message(args, kargs)));
}
