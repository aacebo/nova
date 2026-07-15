use nova_core::{Args, Binding, KArgs, Step, StepBody};
use nova_reflect::Value;

use crate::Scope;

pub(crate) fn invoke(step: &Step, scope: &Scope) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(cond) = &step.cond
        && !scope.eval(cond).map(|v| v.is_truthy()).unwrap_or(false)
    {
        return Ok(());
    }

    match &step.body {
        StepBody::Call { call, args, with } => {
            let resolve = |value: &Binding| -> Value {
                match value.value().as_str() {
                    Some(source) => scope
                        .eval(source)
                        .map(Binding::into_value)
                        .unwrap_or_else(|_| value.clone().into_value()),
                    None => value.clone().into_value(),
                }
            };

            let positional: Vec<Value> = args.iter().map(&resolve).collect();
            let mut kargs = KArgs::new();

            for (key, value) in with {
                kargs.set(key.clone(), resolve(value));
            }

            if let Err(err) = scope.call(call, Args::new(positional, kargs)) {
                scope.error(err.to_string());
            }
        }
        StepBody::Run { run } => {
            if let Err(err) = scope.render_str(run) {
                scope.error(err.to_string());
            }
        }
        StepBody::Shell { shell } => {
            let cmd = match scope.render_str(shell) {
                Ok(cmd) => cmd,
                Err(err) => {
                    scope.error(err.to_string());
                    return Ok(());
                }
            };

            let status = std::process::Command::new(if cfg!(windows) { "cmd" } else { "sh" })
                .arg(if cfg!(windows) { "/C" } else { "-c" })
                .arg(&cmd)
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status();

            match status {
                Ok(status) if !status.success() => {
                    scope.error(format!("shell exited {}", status));
                }
                Ok(_) => {}
                Err(err) => {
                    scope.error(err.to_string());
                }
            }
        }
    }

    Ok(())
}
