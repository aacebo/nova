pub mod object;
pub mod step;

use crate::{Diagnostic, Error};

pub trait Observer: Send + 'static {
    fn on_event(&self, event: &Event) {
        match &event.source {
            Source::Error(err) => self.on_error(err),
            Source::Diagnostic(d) => self.on_diagnostic(d),
            Source::Object(object::ObjectEvent::Call(e)) => self.on_call(e),
            Source::Object(object::ObjectEvent::Update(e)) => self.on_update(e),
            Source::Step(step::StepEvent::Start(e)) => self.on_step_start(e),
            Source::Step(step::StepEvent::End(e)) => self.on_step_end(e),
        }
    }

    fn on_call(&self, _event: &object::CallEvent) {}
    fn on_update(&self, _event: &object::UpdateEvent) {}
    fn on_step_start(&self, _event: &step::StartEvent) {}
    fn on_step_end(&self, _event: &step::EndEvent) {}
    fn on_error(&self, _event: &Error) {}
    fn on_diagnostic(&self, _event: &Diagnostic) {}
}

impl<T> Observer for T
where
    T: Fn(&Event) + Send + 'static,
{
    fn on_event(&self, event: &Event) {
        (self)(event)
    }
}

pub fn on_call<F: Fn(&object::CallEvent) + Send + 'static>(f: F) -> impl Observer {
    struct W<F>(F);

    impl<F: Fn(&object::CallEvent) + Send + 'static> Observer for W<F> {
        fn on_call(&self, event: &object::CallEvent) {
            (self.0)(event)
        }
    }

    W(f)
}

pub fn on_update<F: Fn(&object::UpdateEvent) + Send + 'static>(f: F) -> impl Observer {
    struct W<F>(F);

    impl<F: Fn(&object::UpdateEvent) + Send + 'static> Observer for W<F> {
        fn on_update(&self, event: &object::UpdateEvent) {
            (self.0)(event)
        }
    }

    W(f)
}

pub fn on_step_start<F: Fn(&step::StartEvent) + Send + 'static>(f: F) -> impl Observer {
    struct W<F>(F);

    impl<F: Fn(&step::StartEvent) + Send + 'static> Observer for W<F> {
        fn on_step_start(&self, event: &step::StartEvent) {
            (self.0)(event)
        }
    }

    W(f)
}

pub fn on_step_end<F: Fn(&step::EndEvent) + Send + 'static>(f: F) -> impl Observer {
    struct W<F>(F);

    impl<F: Fn(&step::EndEvent) + Send + 'static> Observer for W<F> {
        fn on_step_end(&self, event: &step::EndEvent) {
            (self.0)(event)
        }
    }

    W(f)
}

pub fn on_error<F: Fn(&Error) + Send + 'static>(f: F) -> impl Observer {
    struct W<F>(F);

    impl<F: Fn(&Error) + Send + 'static> Observer for W<F> {
        fn on_error(&self, event: &Error) {
            (self.0)(event)
        }
    }

    W(f)
}

pub fn on_diagnostic<F: Fn(&Diagnostic) + Send + 'static>(f: F) -> impl Observer {
    struct W<F>(F);

    impl<F: Fn(&Diagnostic) + Send + 'static> Observer for W<F> {
        fn on_diagnostic(&self, event: &Diagnostic) {
            (self.0)(event)
        }
    }

    W(f)
}

pub fn new(trace_id: ulid::Ulid, path: impl Into<String>, source: impl Into<Source>) -> Event {
    Event {
        trace_id,
        path: path.into(),
        source: source.into(),
        timestamp: std::time::SystemTime::now(),
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
    Error(Error),
    Diagnostic(Diagnostic),
    Object(object::ObjectEvent),
    Step(step::StepEvent),
}

impl From<Error> for Source {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

impl From<Diagnostic> for Source {
    fn from(value: Diagnostic) -> Self {
        Self::Diagnostic(value)
    }
}

impl From<object::ObjectEvent> for Source {
    fn from(value: object::ObjectEvent) -> Self {
        Self::Object(value)
    }
}

impl From<step::StepEvent> for Source {
    fn from(value: step::StepEvent) -> Self {
        Self::Step(value)
    }
}
