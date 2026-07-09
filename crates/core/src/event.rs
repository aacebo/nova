use crate::{Error, KArgs, Value};

pub trait Observer: Send + 'static {
    fn on_event(&self, event: Event);
}

impl<T> Observer for T
where
    T: Fn(Event) + Send + 'static,
{
    fn on_event(&self, event: Event) {
        (self)(event)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Event {
    pub trace_id: ulid::Ulid,
    pub path: String,
    pub source: Source,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum Source {
    Error { error: Error },
    Call { name: String, args: Vec<Value>, kargs: KArgs },
    Update { name: String, from: Value, to: Value },
}

impl Source {
    pub fn error(error: Error) -> Self {
        Self::Error { error }
    }

    pub fn call(
        name: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<Value>>,
        kargs: impl IntoIterator<Item = (impl Into<String>, impl Into<Value>)>,
    ) -> Self {
        Self::Call {
            name: name.into(),
            args: args.into_iter().map(|v| v.into()).collect(),
            kargs: kargs.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
        }
    }

    pub fn update(name: impl Into<String>, from: impl Into<Value>, to: impl Into<Value>) -> Self {
        Self::Update {
            name: name.into(),
            from: from.into(),
            to: to.into(),
        }
    }
}
