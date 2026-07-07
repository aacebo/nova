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
