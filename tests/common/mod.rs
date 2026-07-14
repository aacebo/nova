#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use nova::event::step::{EndEvent, StartEvent};
use nova::template::Pointer;
use nova::{Diagnostic, Event, Observer, Severity};

#[derive(Clone, Default)]
pub struct Recorder(Arc<Inner>);

#[derive(Default)]
struct Inner {
    calls: Mutex<Vec<String>>,
    updates: Mutex<Vec<(String, Pointer, Pointer)>>,
    errors: Mutex<Vec<String>>,
    diagnostics: Mutex<Vec<Diagnostic>>,
    events: Mutex<usize>,
    step_starts: Mutex<Vec<StartEvent>>,
    step_ends: Mutex<Vec<EndEvent>>,
}

impl Recorder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calls(&self) -> Vec<String> {
        self.0.calls.lock().unwrap().clone()
    }

    pub fn updates(&self) -> Vec<(String, Pointer, Pointer)> {
        self.0.updates.lock().unwrap().clone()
    }

    pub fn errors(&self) -> Vec<String> {
        self.0.errors.lock().unwrap().clone()
    }

    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.0.diagnostics.lock().unwrap().clone()
    }

    pub fn messages(&self) -> Vec<String> {
        self.0
            .diagnostics
            .lock()
            .unwrap()
            .iter()
            .filter_map(|d| d.message.clone())
            .collect()
    }

    pub fn has_error(&self) -> bool {
        self.0
            .diagnostics
            .lock()
            .unwrap()
            .iter()
            .any(|d| d.severity() == Severity::Error)
    }

    pub fn event_count(&self) -> usize {
        *self.0.events.lock().unwrap()
    }

    pub fn step_starts(&self) -> Vec<StartEvent> {
        self.0.step_starts.lock().unwrap().clone()
    }

    pub fn step_ends(&self) -> Vec<EndEvent> {
        self.0.step_ends.lock().unwrap().clone()
    }
}

impl Observer for Recorder {
    fn on_event(&self, event: &Event) {
        *self.0.events.lock().unwrap() += 1;
        self.dispatch(event);
    }
}

impl Recorder {
    fn dispatch(&self, event: &Event) {
        use nova::event::Source;
        use nova::event::object::ObjectEvent;
        use nova::event::step::StepEvent;

        match &event.source {
            Source::Error(err) => self.0.errors.lock().unwrap().push(err.to_string()),
            Source::Diagnostic(d) => self.0.diagnostics.lock().unwrap().push(d.clone()),
            Source::Object(ObjectEvent::Call(e)) => self.0.calls.lock().unwrap().push(e.name.clone()),
            Source::Object(ObjectEvent::Update(e)) => {
                self.0
                    .updates
                    .lock()
                    .unwrap()
                    .push((e.name.clone(), e.from.clone(), e.to.clone()))
            }
            Source::Step(StepEvent::Start(e)) => self.0.step_starts.lock().unwrap().push(e.clone()),
            Source::Step(StepEvent::End(e)) => self.0.step_ends.lock().unwrap().push(e.clone()),
        }
    }
}

pub fn to_pointer<T: nova::reflect::ToValue>(value: T) -> Pointer {
    Pointer::new(value.to_value().into_owned())
}
