use nova_core::{Args, Binding, Context, Diagnostic, Error, FromArgs, Severity};
use nova_reflect::Value;

use crate::Builder;

pub struct EnvArgs {
    pub name: String,
    pub default: Binding,
}

impl FromArgs for EnvArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &Args) -> Result<Self, Self::Error> {
        let name = args.str(0).ok_or(Error::message("name must be a string"))?;

        Ok(Self {
            name,
            default: Binding::Value(args.key("default")),
        })
    }
}

pub struct FormatArgs {
    pub message: Value,
}

impl FormatArgs {
    pub fn text(&self) -> String {
        if self.message.is_undefined() || self.message.is_null() {
            String::new()
        } else {
            self.message.to_string()
        }
    }
}

impl FromArgs for FormatArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &Args) -> Result<Self, Self::Error> {
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
        .func("env", |args: &Args, _ctx: &dyn Context| {
            let args = EnvArgs::from_args(args)?;

            match std::env::var(&args.name) {
                Ok(value) => Ok(Binding::new(Value::from(value))),
                Err(_) => Ok(args.default),
            }
        })
        .func("info", |args: &Args, ctx: &dyn Context| {
            emit(Severity::Info, args, ctx)?;
            Ok(Binding::new(Value::Null))
        })
        .func("warn", |args: &Args, ctx: &dyn Context| {
            emit(Severity::Warn, args, ctx)?;
            Ok(Binding::new(Value::Null))
        })
        .func("error", |args: &Args, ctx: &dyn Context| {
            emit(Severity::Error, args, ctx)?;
            Ok(Binding::new(Value::Null))
        })
        .func("print", |args: &Args, _ctx: &dyn Context| {
            print!("{}", FormatArgs::from_args(args)?.text());
            Ok(Binding::new(Value::Null))
        })
        .func("println", |args: &Args, _ctx: &dyn Context| {
            println!("{}", FormatArgs::from_args(args)?.text());
            Ok(Binding::new(Value::Null))
        })
}

fn emit(severity: Severity, args: &Args, ctx: &dyn Context) -> Result<(), Box<dyn std::error::Error>> {
    let args = FormatArgs::from_args(args)?;
    ctx.emit(Diagnostic::new(ctx.trace_id()).sev(severity).message(args.text()));
    Ok(())
}
