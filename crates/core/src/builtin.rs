use crate::{Builder, Diagnostic, KArgs, Severity, Value};

pub fn register(builder: Builder) -> Builder {
    builder
        .func("info", |args: &[Value], kargs: &KArgs| {
            emit(Severity::Info, args, kargs);
            Ok(None)
        })
        .func("warn", |args: &[Value], kargs: &KArgs| {
            emit(Severity::Warn, args, kargs);
            Ok(None)
        })
        .func("error", |args: &[Value], kargs: &KArgs| {
            emit(Severity::Error, args, kargs);
            Ok(None)
        })
        .func("print", |args: &[Value], kargs: &KArgs| {
            print!("{}", message(args, kargs));
            Ok(None)
        })
        .func("println", |args: &[Value], kargs: &KArgs| {
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

fn emit(severity: Severity, args: &[Value], kargs: &KArgs) {
    Diagnostic::new(crate::trace_id())
        .sev(severity)
        .message(message(args, kargs))
        .emit();
}
