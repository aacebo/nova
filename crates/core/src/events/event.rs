use crate::{Error, KArgs, Object, Value};

#[derive(Debug, Clone, serde::Serialize)]
pub struct Event<'a> {
    pub trace_id: ulid::Ulid,
    pub path: String,
    #[serde(flatten)]
    pub source: Source<'a>,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum Source<'a> {
    Error(&'a Error),
    Object(ObjectEvent<'a>),
}

impl<'a> From<ObjectEvent<'a>> for Source<'a> {
    fn from(value: ObjectEvent<'a>) -> Self {
        Self::Object(value)
    }
}

#[derive(Debug, Copy, Clone, serde::Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ObjectEvent<'a> {
    Call {
        args: &'a [Value],
        kargs: &'a KArgs,
        object: &'a Object,
    },
    Create(&'a Object),
    Update(&'a Object),
    Delete(&'a Object),
}
