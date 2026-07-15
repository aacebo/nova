use std::collections::BTreeMap;

use crate::Binding;

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum StepBody {
    Call {
        call: String,

        #[serde(default)]
        args: Vec<Binding>,

        #[serde(default)]
        with: BTreeMap<String, Binding>,
    },
    Run {
        run: String,
    },
    Shell {
        shell: String,
    },
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
            args: impl IntoIterator<Item = impl Into<Binding>>,
            with: impl IntoIterator<Item = (impl Into<String>, impl Into<Binding>)>,
        ) -> Self {
            self.body = Some(StepBody::Call {
                call: name.into(),
                args: args.into_iter().map(Into::into).collect(),
                with: with.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
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
