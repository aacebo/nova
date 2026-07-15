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
        .func("env", |ctx: &dyn Context| {
            let args = EnvArgs::from_args(ctx.args())?;

            match std::env::var(&args.name) {
                Ok(value) => Ok(Binding::new(Value::from(value))),
                Err(_) => Ok(args.default),
            }
        })
        .func("info", |ctx: &dyn Context| {
            emit(Severity::Info, ctx)?;
            Ok(Binding::new(Value::Null))
        })
        .func("warn", |ctx: &dyn Context| {
            emit(Severity::Warn, ctx)?;
            Ok(Binding::new(Value::Null))
        })
        .func("error", |ctx: &dyn Context| {
            emit(Severity::Error, ctx)?;
            Ok(Binding::new(Value::Null))
        })
        .func("print", |ctx: &dyn Context| {
            print!("{}", FormatArgs::from_args(ctx.args())?.text());
            Ok(Binding::new(Value::Null))
        })
        .func("println", |ctx: &dyn Context| {
            println!("{}", FormatArgs::from_args(ctx.args())?.text());
            Ok(Binding::new(Value::Null))
        })
}

fn emit(severity: Severity, ctx: &dyn Context) -> Result<(), Box<dyn std::error::Error>> {
    let args = FormatArgs::from_args(ctx.args())?;
    ctx.emit(Diagnostic::new(ctx.trace_id()).sev(severity).message(args.text()));
    Ok(())
}
