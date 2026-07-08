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
    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub on: Vec<Trigger>,

    #[serde(default)]
    pub vars: BTreeMap<String, Value>,

    #[serde(default)]
    pub templates: BTreeMap<String, String>,

    #[serde(default)]
    pub steps: Vec<Step>,
}

impl TryFrom<Manifest> for Runtime {
    type Error = Box<dyn std::error::Error>;

    fn try_from(manifest: Manifest) -> Result<Self, Self::Error> {
        crate::load(manifest)?.build()
    }
}

impl TryFrom<Vec<Manifest>> for Runtime {
    type Error = Box<dyn std::error::Error>;

    fn try_from(manifests: Vec<Manifest>) -> Result<Self, Self::Error> {
        crate::load_all(manifests)?.build()
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

        pub fn on(mut self, value: impl IntoIterator<Item = Trigger>) -> Self {
            self.on.extend(value);
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

    impl From<ManifestBuilder> for Manifest {
        fn from(value: ManifestBuilder) -> Self {
            value.build()
        }
    }
}
