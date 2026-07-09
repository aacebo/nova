use super::Source;

pub fn start(task: impl Into<String>, name: impl Into<String>, index: impl Into<usize>, total: impl Into<usize>) -> StartEvent {
    StartEvent {
        task: task.into(),
        name: name.into(),
        index: index.into(),
        total: total.into(),
    }
}

pub fn end(
    task: impl Into<String>,
    name: impl Into<String>,
    index: impl Into<usize>,
    status: impl Into<Status>,
    elapsed: impl Into<std::time::Duration>,
) -> EndEvent {
    EndEvent {
        task: task.into(),
        name: name.into(),
        index: index.into(),
        status: status.into(),
        elapsed: elapsed.into(),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ok,
    Skipped,
    Error,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum StepEvent {
    Start(StartEvent),
    End(EndEvent),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StartEvent {
    pub task: String,
    pub name: String,
    pub index: usize,
    pub total: usize,
}

impl From<StartEvent> for Source {
    fn from(value: StartEvent) -> Self {
        Self::Step(value.into())
    }
}

impl From<StartEvent> for StepEvent {
    fn from(value: StartEvent) -> Self {
        Self::Start(value)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EndEvent {
    pub task: String,
    pub name: String,
    pub index: usize,
    pub status: Status,
    pub elapsed: std::time::Duration,
}

impl From<EndEvent> for Source {
    fn from(value: EndEvent) -> Self {
        Self::Step(value.into())
    }
}

impl From<EndEvent> for StepEvent {
    fn from(value: EndEvent) -> Self {
        Self::End(value)
    }
}
