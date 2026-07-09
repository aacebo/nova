use super::Source;
use crate::{KArgs, Value};

pub fn call(
    name: impl Into<String>,
    args: impl IntoIterator<Item = impl Into<Value>>,
    kargs: impl IntoIterator<Item = (impl Into<String>, impl Into<Value>)>,
) -> ObjectEvent {
    ObjectEvent::Call(CallEvent {
        name: name.into(),
        args: args.into_iter().map(|v| v.into()).collect(),
        kargs: kargs.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
    })
}

pub fn update(name: impl Into<String>, from: impl Into<Value>, to: impl Into<Value>) -> ObjectEvent {
    ObjectEvent::Update(UpdateEvent {
        name: name.into(),
        from: from.into(),
        to: to.into(),
    })
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ObjectEvent {
    Call(CallEvent),
    Update(UpdateEvent),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CallEvent {
    pub name: String,
    pub args: Vec<Value>,
    pub kargs: KArgs,
}

impl From<CallEvent> for Source {
    fn from(value: CallEvent) -> Self {
        Self::Object(value.into())
    }
}

impl From<CallEvent> for ObjectEvent {
    fn from(value: CallEvent) -> Self {
        Self::Call(value)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UpdateEvent {
    pub name: String,
    pub from: Value,
    pub to: Value,
}

impl From<UpdateEvent> for Source {
    fn from(value: UpdateEvent) -> Self {
        Self::Object(value.into())
    }
}

impl From<UpdateEvent> for ObjectEvent {
    fn from(value: UpdateEvent) -> Self {
        Self::Update(value)
    }
}
