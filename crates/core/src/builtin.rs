use nova_reflect::Value;
use nova_template::{Args, FromArgs, Pointer};

use crate::{Builder, Diagnostic, Error, Scope, Severity};

pub struct EnvArgs {
    pub name: String,
    pub default: Pointer,
}

impl FromArgs for EnvArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &Args) -> Result<Self, Self::Error> {
        let name = args
            .at(0)
            .value()
            .as_str()
            .map(|s| s.to_string())
            .ok_or(Error::message("name must be a string"))?;

        Ok(Self {
            name,
            default: args.key("default"),
        })
    }
}

pub struct FormatArgs {
    pub message: Pointer,
}

impl FormatArgs {
    pub fn text(&self) -> String {
        let value = self.message.value();

        if value.is_undefined() || value.is_null() {
            String::new()
        } else {
            value.to_string()
        }
    }
}

impl FromArgs for FormatArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &Args) -> Result<Self, Self::Error> {
        let primary = args.at(0);

        Ok(Self {
            message: if primary.value().is_undefined() {
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
                Ok(value) => Ok(Pointer::new(Value::from(value))),
                Err(_) => Ok(args.default),
            }
        })
        .func("info", |args: &Args, scope: &Scope| {
            emit(Severity::Info, args, scope)?;
            Ok(Pointer::new(Value::Null))
        })
        .func("warn", |args: &Args, scope: &Scope| {
            emit(Severity::Warn, args, scope)?;
            Ok(Pointer::new(Value::Null))
        })
        .func("error", |args: &Args, scope: &Scope| {
            emit(Severity::Error, args, scope)?;
            Ok(Pointer::new(Value::Null))
        })
        .func("print", |args: &Args, _scope: &Scope| {
            print!("{}", FormatArgs::from_args(args)?.text());
            Ok(Pointer::new(Value::Null))
        })
        .func("println", |args: &Args, _scope: &Scope| {
            println!("{}", FormatArgs::from_args(args)?.text());
            Ok(Pointer::new(Value::Null))
        })
}

fn emit(severity: Severity, args: &Args, scope: &Scope) -> Result<(), Box<dyn std::error::Error>> {
    let args = FormatArgs::from_args(args)?;
    scope.emit(Diagnostic::new(*scope.trace_id()).sev(severity).message(args.text()));
    Ok(())
}
