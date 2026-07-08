use crate::{Builder, Diagnostic, KArgs, Scope, Severity, Value};

pub fn register(builder: Builder) -> Builder {
    builder
        .func("info", |args: &[Value], kargs: &KArgs, scope: &Scope| {
            emit(Severity::Info, args, kargs, scope);
            Ok(None)
        })
        .func("warn", |args: &[Value], kargs: &KArgs, scope: &Scope| {
            emit(Severity::Warn, args, kargs, scope);
            Ok(None)
        })
        .func("error", |args: &[Value], kargs: &KArgs, scope: &Scope| {
            emit(Severity::Error, args, kargs, scope);
            Ok(None)
        })
        .func("print", |args: &[Value], kargs: &KArgs, _scope: &Scope| {
            print!("{}", message(args, kargs));
            Ok(None)
        })
        .func("println", |args: &[Value], kargs: &KArgs, _scope: &Scope| {
            println!("{}", message(args, kargs));
            Ok(None)
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
