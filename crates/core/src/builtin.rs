use crate::{Args, Builder, Diagnostic, Error, FromArgs, Scope, Severity, Value};

pub struct EnvArgs {
    pub name: String,
    pub default: Value,
}

impl FromArgs for EnvArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &Args<'_>) -> Result<Self, Self::Error> {
        let name = args.at(0);
        let name = name.as_str().ok_or(Error::message("name must be a string"))?;

        Ok(Self {
            name: name.to_string(),
            default: args.key("default"),
        })
    }
}

pub struct FormatArgs {
    pub message: Value,
}

impl FormatArgs {
    pub fn text(&self) -> String {
        if self.message.is_undefined() {
            String::new()
        } else {
            self.message.to_string()
        }
    }
}

impl FromArgs for FormatArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &Args<'_>) -> Result<Self, Self::Error> {
        let primary = args.at(0);

        Ok(Self {
            message: if primary.is_undefined() {
                args.key("message")
            } else {
                primary
            },
        })
    }
}

pub fn register(builder: Builder) -> Builder {
    builder
        .func("env", |args: &Args, _scope: &Scope| {
            let args = EnvArgs::from_args(args)?;

            match std::env::var(&args.name) {
                Ok(value) => Ok(Value::from(value)),
                Err(_) => Ok(args.default),
            }
        })
        .func("info", |args: &Args, scope: &Scope| {
            emit(Severity::Info, args, scope)?;
            Ok(Value::from(()))
        })
        .func("warn", |args: &Args, scope: &Scope| {
            emit(Severity::Warn, args, scope)?;
            Ok(Value::from(()))
        })
        .func("error", |args: &Args, scope: &Scope| {
            emit(Severity::Error, args, scope)?;
            Ok(Value::from(()))
        })
        .func("print", |args: &Args, _scope: &Scope| {
            print!("{}", FormatArgs::from_args(args)?.text());
            Ok(Value::from(()))
        })
        .func("println", |args: &Args, _scope: &Scope| {
            println!("{}", FormatArgs::from_args(args)?.text());
            Ok(Value::from(()))
        })
}

fn emit(severity: Severity, args: &Args, scope: &Scope) -> Result<(), Box<dyn std::error::Error>> {
    let args = FormatArgs::from_args(args)?;
    scope.emit(Diagnostic::new(*scope.trace_id()).sev(severity).message(args.text()));
    Ok(())
}
