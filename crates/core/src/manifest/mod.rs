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
    pub args: Option<Value>,

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
        self.args = merge_schemas(self.args.take(), other.args);
        self.vars.extend(other.vars);
        self.env.extend(other.env);
        self.templates.extend(other.templates);
        self.steps.extend(other.steps);
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

fn merge_schemas(base: Option<Value>, next: Option<Value>) -> Option<Value> {
    match (base, next) {
        (Some(base), Some(next)) => {
            let mut all_of = into_all_of(base);
            all_of.append(&mut into_all_of(next));
            Some(Value::from_serialize(serde_json::json!({ "allOf": all_of })))
        }
        (base, next) => base.or(next),
    }
}

fn into_all_of(schema: Value) -> Vec<serde_json::Value> {
    let schema: serde_json::Value = serde_json::to_value(&schema).unwrap_or(serde_json::Value::Bool(true));

    match schema {
        serde_json::Value::Object(mut map) if map.len() == 1 && map.contains_key("allOf") => match map.remove("allOf") {
            Some(serde_json::Value::Array(items)) => items,
            other => vec![other.unwrap_or(serde_json::Value::Bool(true))],
        },
        other => vec![other],
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
        args: Option<Value>,
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

        pub fn args(mut self, schema: impl Into<Value>) -> Self {
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
