use std::collections::BTreeMap;

use crate::{Action, KArgs, Value, emit, scope};

pub fn step() -> build::StepBuilder {
    build::StepBuilder::new()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Step {
    #[serde(default, rename = "if")]
    pub cond: Option<String>,

    #[serde(default)]
    pub name: Option<String>,

    #[serde(flatten)]
    pub body: StepBody,
}

impl std::ops::Deref for Step {
    type Target = StepBody;

    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

impl std::ops::DerefMut for Step {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.body
    }
}

impl Action for Step {
    fn invoke(&self, _args: &[Value], _kargs: &KArgs) -> Result<(), Box<dyn std::error::Error>> {
        let scope = scope();

        if let Some(cond) = &self.cond
            && !scope.eval(cond).map(|v| v.is_true()).unwrap_or(false)
        {
            return Ok(());
        }

        match &self.body {
            StepBody::Call { call, args: fields } => {
                let mut kargs = KArgs::new();

                for (key, value) in fields {
                    let value = match value.as_str() {
                        Some(source) => scope.eval(source).unwrap_or_else(|_| value.clone()),
                        None => value.clone(),
                    };

                    kargs.set(key.clone(), value);
                }

                if let Err(err) = scope.call(call, Vec::new(), kargs) {
                    emit([err.to_string()]);
                }
            }
            StepBody::Run { run } => {
                if let Err(err) = scope.render_str(run) {
                    emit([err.to_string()]);
                }
            }
            StepBody::Shell { shell } => {
                let cmd = match scope.render_str(shell) {
                    Ok(cmd) => cmd,
                    Err(err) => {
                        emit([err.to_string()]);
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
                        emit([format!("shell exited {}", status)]);
                    }
                    Ok(_) => {}
                    Err(err) => emit([err.to_string()]),
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum StepBody {
    Call { call: String, args: BTreeMap<String, Value> },
    Run { run: String },
    Shell { shell: String },
}

#[doc(hidden)]
pub mod build {
    use super::*;

    #[derive(Debug, Default, Clone)]
    pub struct StepBuilder {
        cond: Option<String>,
        name: Option<String>,
        body: Option<StepBody>,
    }

    impl StepBuilder {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn guard(mut self, value: impl Into<String>) -> Self {
            self.cond = Some(value.into());
            self
        }

        pub fn name(mut self, value: impl Into<String>) -> Self {
            self.name = Some(value.into());
            self
        }

        pub fn run(mut self, name: impl Into<String>) -> Self {
            self.body = Some(StepBody::Run { run: name.into() });
            self
        }

        pub fn shell(mut self, cmd: impl Into<String>) -> Self {
            self.body = Some(StepBody::Shell { shell: cmd.into() });
            self
        }

        pub fn call(
            mut self,
            name: impl Into<String>,
            args: impl IntoIterator<Item = (impl Into<String>, impl Into<Value>)>,
        ) -> Self {
            self.body = Some(StepBody::Call {
                call: name.into(),
                args: args.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
            });

            self
        }

        pub fn build(self) -> Step {
            Step {
                cond: self.cond,
                name: self.name,
                body: self.body.expect("step must have content"),
            }
        }
    }

    impl From<StepBuilder> for Step {
        fn from(value: StepBuilder) -> Self {
            value.build()
        }
    }
}
