use std::collections::BTreeMap;

use crate::Value;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub on: Vec<String>,

    #[serde(default)]
    pub vars: BTreeMap<String, Value>,

    #[serde(default)]
    pub templates: BTreeMap<String, String>,

    #[serde(default)]
    pub steps: Vec<Step>,
}

impl Manifest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> build::ManifestBuilder {
        build::ManifestBuilder::new()
    }
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
    Call { call: String, args: BTreeMap<String, Value> },
    Run { run: String },
}

#[doc(hidden)]
pub mod build {
    use super::*;

    #[derive(Debug, Default, Clone)]
    pub struct ManifestBuilder {
        name: Option<String>,
        on: Vec<String>,
        vars: BTreeMap<String, Value>,
        templates: BTreeMap<String, String>,
        steps: Vec<Step>,
    }

    impl ManifestBuilder {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn name(mut self, value: impl Into<String>) -> Self {
            self.name = Some(value.into());
            self
        }

        pub fn on(mut self, value: impl IntoIterator<Item = impl Into<String>>) -> Self {
            self.on.extend(value.into_iter().map(|v| v.into()));
            self
        }

        pub fn var(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
            self.vars.insert(name.into(), value.into());
            self
        }

        pub fn vars(mut self, value: impl IntoIterator<Item = (impl Into<String>, impl Into<Value>)>) -> Self {
            self.vars.extend(value.into_iter().map(|(k, v)| (k.into(), v.into())));
            self
        }

        pub fn template(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
            self.templates.insert(name.into(), value.into());
            self
        }

        pub fn templates(mut self, value: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
            self.templates.extend(value.into_iter().map(|(k, v)| (k.into(), v.into())));
            self
        }

        pub fn step(mut self, value: impl Into<Step>) -> Self {
            self.steps.push(value.into());
            self
        }

        pub fn steps(mut self, value: impl IntoIterator<Item = impl Into<Step>>) -> Self {
            self.steps.extend(value.into_iter().map(|s| s.into()));
            self
        }

        pub fn build(self) -> Manifest {
            Manifest {
                name: self.name,
                on: self.on,
                vars: self.vars,
                templates: self.templates,
                steps: self.steps,
            }
        }
    }

    impl Into<Manifest> for ManifestBuilder {
        fn into(self) -> Manifest {
            self.build()
        }
    }

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

    impl Into<Step> for StepBuilder {
        fn into(self) -> Step {
            self.build()
        }
    }
}
