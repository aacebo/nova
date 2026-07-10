use crate::{Args, Builder, Diagnostic, Error, Scope, Severity, Value};

pub fn register(builder: Builder) -> Builder {
    builder
        .func("env", |args: &Args, _scope: &Scope| {
            let key = args.at(0);
            let key = key.as_str().ok_or(Error::message("name must be a string"))?;

            match std::env::var(key) {
                Ok(value) => Ok(Value::from(value)),
                Err(_) => Ok(args.key("default")),
            }
        })
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
