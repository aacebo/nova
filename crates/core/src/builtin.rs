use crate::{Args, Builder, Diagnostic, Scope, Severity, Value};

pub fn register(builder: Builder) -> Builder {
    builder
        .func("info", |args: &Args, scope: &Scope| {
            emit(Severity::Info, args, scope);
            Ok(Value::from(()))
        })
        .func("warn", |args: &Args, scope: &Scope| {
            emit(Severity::Warn, args, scope);
            Ok(Value::from(()))
        })
        .func("error", |args: &Args, scope: &Scope| {
            emit(Severity::Error, args, scope);
            Ok(Value::from(()))
        })
        .func("print", |args: &Args, _scope: &Scope| {
            print!("{}", message(args));
            Ok(Value::from(()))
        })
        .func("println", |args: &Args, _scope: &Scope| {
            println!("{}", message(args));
            Ok(Value::from(()))
        })
}

fn message(args: &Args) -> String {
    let primary = args.at(0);
    let value = if primary.is_undefined() {
        args.key("message")
    } else {
        primary
    };

    if value.is_undefined() {
        String::new()
    } else {
        value.to_string()
    }
}

fn emit(severity: Severity, args: &Args, scope: &Scope) {
    scope.emit(Diagnostic::new(*scope.trace_id()).sev(severity).message(message(args)));
}
