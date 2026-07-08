use crate::{Args, Builder, Diagnostic, Severity};

pub fn register(builder: Builder) -> Builder {
    builder
        .func("info", |args: &Args| {
            emit(Severity::Info, args);
            Ok(None)
        })
        .func("warn", |args: &Args| {
            emit(Severity::Warn, args);
            Ok(None)
        })
        .func("error", |args: &Args| {
            emit(Severity::Error, args);
            Ok(None)
        })
        .func("print", |args: &Args| {
            print!("{}", message(args));
            Ok(None)
        })
        .func("println", |args: &Args| {
            println!("{}", message(args));
            Ok(None)
        })
}

fn message(args: &Args) -> String {
    args.at(0)
        .or_else(|| args.get("message"))
        .map(|v| v.to_string())
        .unwrap_or_default()
}

fn emit(severity: Severity, args: &Args) {
    Diagnostic::new(crate::trace_id()).sev(severity).message(message(args)).emit();
}
