mod step;
mod trigger;

use std::collections::BTreeMap;

pub use step::*;
pub use trigger::*;

use crate::{Runtime, Value};

pub fn manifest() -> build::ManifestBuilder {
    build::ManifestBuilder::new()
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub name: String,

    #[serde(default)]
    pub on: Vec<Trigger>,

    #[serde(default)]
    pub include: Vec<String>,

    #[serde(default)]
    pub args: Option<nova_schema::Schema>,

    #[serde(default)]
    pub vars: BTreeMap<String, Value>,

    #[serde(default)]
    pub env: BTreeMap<String, String>,

    #[serde(default)]
    pub templates: BTreeMap<String, String>,

    #[serde(default)]
    pub steps: Vec<Step>,
}

impl Manifest {
    pub fn merge(&mut self, other: Self) -> &mut Self {
        self.on.extend(other.on);
        self.vars.extend(other.vars);
        self.env.extend(other.env);
        self.templates.extend(other.templates);
        self.steps.extend(other.steps);
        self.args = match (self.args.take(), other.args) {
            (Some(base), Some(next)) => Some(nova_schema::oneof!(base, next).into()),
            (base, next) => base.or(next),
        };

        self
    }
}

impl TryFrom<Manifest> for Runtime {
    type Error = Box<dyn std::error::Error>;

    fn try_from(manifest: Manifest) -> Result<Self, Self::Error> {
        crate::new().routine(manifest).build()
    }
}

impl TryFrom<Vec<Manifest>> for Runtime {
    type Error = Box<dyn std::error::Error>;

    fn try_from(manifests: Vec<Manifest>) -> Result<Self, Self::Error> {
        let mut builder = crate::new();

        for manifest in manifests {
            builder = builder.routine(manifest);
        }

        builder.build()
    }
}

#[doc(hidden)]
pub mod build {
    pub use step::build::StepBuilder;

    use super::*;

    #[derive(Debug, Default, Clone)]
    pub struct ManifestBuilder {
        name: Option<String>,
        on: Vec<Trigger>,
        include: Vec<String>,
        args: Option<nova_schema::Schema>,
        vars: BTreeMap<String, Value>,
        env: BTreeMap<String, String>,
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

        pub fn on(mut self, value: impl IntoIterator<Item = Trigger>) -> Self {
            self.on.extend(value);
            self
        }

        pub fn include(mut self, value: impl IntoIterator<Item = impl Into<String>>) -> Self {
            self.include.extend(value.into_iter().map(Into::into));
            self
        }

        pub fn args(mut self, schema: impl Into<nova_schema::Schema>) -> Self {
            self.args = Some(schema.into());
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

        pub fn env(mut self, name: impl Into<String>, var: impl Into<String>) -> Self {
            self.env.insert(name.into(), var.into());
            self
        }

        pub fn envs(mut self, value: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
            self.env.extend(value.into_iter().map(|(k, v)| (k.into(), v.into())));
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
                name: self.name.expect("manifest requires a `name`"),
                on: self.on,
                include: self.include,
                args: self.args,
                vars: self.vars,
                env: self.env,
                templates: self.templates,
                steps: self.steps,
            }
        }
    }

    impl From<ManifestBuilder> for Manifest {
        fn from(value: ManifestBuilder) -> Self {
            value.build()
        }
    }
}
