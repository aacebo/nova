use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use nova::event::step::{EndEvent, StartEvent, Status};
use nova::{Diagnostic, Severity};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum State {
    Pending,
    Running,
    Ok,
    Skipped,
    Error,
}

impl From<Status> for State {
    fn from(value: Status) -> Self {
        match value {
            Status::Ok => Self::Ok,
            Status::Skipped => Self::Skipped,
            Status::Error => Self::Error,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StepRow {
    pub name: String,
    pub state: State,
    pub elapsed: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct TaskRow {
    pub name: String,
    pub state: State,
    pub steps: Vec<StepRow>,
    pub started: Option<Instant>,
    pub elapsed: Option<Duration>,
}

impl TaskRow {
    fn step_mut(&mut self, index: usize, name: &str) -> &mut StepRow {
        if index >= self.steps.len() {
            self.steps.resize_with(index + 1, || StepRow {
                name: String::new(),
                state: State::Pending,
                elapsed: None,
            });
        }

        let step = &mut self.steps[index];

        if step.name.is_empty() {
            step.name = if name.is_empty() {
                format!("step {}", index + 1)
            } else {
                name.to_string()
            };
        }

        step
    }
}

#[derive(Debug, Default)]
pub struct BoardState {
    pub tasks: Vec<TaskRow>,
    pub started: Option<Instant>,
}

impl BoardState {
    fn task_mut(&mut self, name: &str) -> &mut TaskRow {
        if let Some(index) = self.tasks.iter().position(|t| t.name == name) {
            return &mut self.tasks[index];
        }

        self.tasks.push(TaskRow {
            name: name.to_string(),
            state: State::Pending,
            steps: Vec::new(),
            started: None,
            elapsed: None,
        });

        self.tasks.last_mut().unwrap()
    }
}

#[derive(Clone)]
pub struct Board(Arc<Mutex<BoardState>>);

impl Board {
    pub fn new(tasks: &[String]) -> Self {
        let tasks = tasks
            .iter()
            .map(|name| TaskRow {
                name: name.clone(),
                state: State::Pending,
                steps: Vec::new(),
                started: None,
                elapsed: None,
            })
            .collect();

        Self(Arc::new(Mutex::new(BoardState {
            tasks,
            started: Some(Instant::now()),
        })))
    }

    pub fn state(&self) -> std::sync::MutexGuard<'_, BoardState> {
        self.0.lock().unwrap()
    }
}

impl nova::Observer for Board {
    fn on_step_start(&self, event: &StartEvent) {
        let mut state = self.0.lock().unwrap();
        let task = state.task_mut(&event.task);

        if task.started.is_none() {
            task.started = Some(Instant::now());
        }

        task.state = State::Running;
        task.step_mut(event.index, &event.name).state = State::Running;
    }

    fn on_step_end(&self, event: &EndEvent) {
        let mut state = self.0.lock().unwrap();
        let task = state.task_mut(&event.task);
        let step = task.step_mut(event.index, &event.name);

        if step.state != State::Error {
            step.state = event.status.into();
        }

        step.elapsed = Some(event.elapsed);

        let done = task
            .steps
            .iter()
            .all(|s| s.state != State::Pending && s.state != State::Running);

        if done {
            task.state = if task.steps.iter().any(|s| s.state == State::Error) {
                State::Error
            } else {
                State::Ok
            };
            task.elapsed = task.started.map(|s| s.elapsed());
        }
    }

    fn on_diagnostic(&self, event: &Diagnostic) {
        if event.severity() != Severity::Error {
            return;
        }

        let mut state = self.0.lock().unwrap();

        for task in state.tasks.iter_mut() {
            if let Some(step) = task.steps.iter_mut().find(|s| s.state == State::Running) {
                step.state = State::Error;
                return;
            }
        }
    }
}

pub struct Widget<'a> {
    state: &'a BoardState,
    tick: usize,
    final_frame: bool,
}

impl<'a> Widget<'a> {
    pub fn new(state: &'a BoardState, tick: usize, final_frame: bool) -> Self {
        Self {
            state,
            tick,
            final_frame,
        }
    }

    pub fn lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        for task in &self.state.tasks {
            lines.push(self.row(&task.name, task.state, task.elapsed, false));

            for step in &task.steps {
                lines.push(self.row(&step.name, step.state, step.elapsed, true));
            }
        }

        if self.final_frame {
            lines.push(self.summary());
        }

        lines
    }

    pub fn height(&self) -> u16 {
        self.lines().len() as u16
    }

    pub fn width(&self) -> u16 {
        self.lines().iter().map(|l| l.width() as u16).max().unwrap_or(0)
    }

    fn row(&self, name: &str, state: State, elapsed: Option<Duration>, indent: bool) -> Line<'static> {
        let dim = Style::default().add_modifier(Modifier::DIM);
        let mut spans = Vec::new();

        if indent {
            spans.push(Span::styled("  ", dim));
        }

        spans.push(Span::styled(
            format!("{} ", self.glyph(state)),
            Style::default().fg(color(state)).bold(),
        ));
        spans.push(Span::raw(name.to_string()));

        if state == State::Skipped {
            spans.push(Span::styled("  (skipped)", dim));
        } else if let Some(elapsed) = elapsed {
            spans.push(Span::styled(format!("  {}", fmt_duration(elapsed)), dim));
        }

        Line::from(spans)
    }

    fn glyph(&self, state: State) -> String {
        match state {
            State::Pending => "○".to_string(),
            State::Running => SPINNER[self.tick % SPINNER.len()].to_string(),
            State::Ok => "✓".to_string(),
            State::Skipped => "○".to_string(),
            State::Error => "✗".to_string(),
        }
    }

    fn summary(&self) -> Line<'static> {
        let mut tasks = 0;
        let mut steps = 0;
        let mut ok = 0;
        let mut skipped = 0;
        let mut failed = 0;

        for task in &self.state.tasks {
            tasks += 1;

            for step in &task.steps {
                steps += 1;

                match step.state {
                    State::Ok => ok += 1,
                    State::Skipped => skipped += 1,
                    State::Error => failed += 1,
                    _ => {}
                }
            }
        }

        let total = self.state.started.map(|s| s.elapsed()).unwrap_or_default();
        let text = format!(
            "{tasks} tasks · {steps} steps · {ok} ok · {skipped} skipped · {failed} failed · total {}",
            fmt_duration(total)
        );

        Line::from(Span::styled(text, Style::default().add_modifier(Modifier::DIM).bold()))
    }
}

impl ratatui::widgets::Widget for &Widget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (i, line) in self.lines().into_iter().enumerate() {
            let y = area.y.saturating_add(i as u16);

            if y >= area.bottom() {
                break;
            }

            buf.set_line(area.x, y, &line, area.width);
        }
    }
}

fn color(state: State) -> Color {
    match state {
        State::Pending => Color::DarkGray,
        State::Running => Color::Cyan,
        State::Ok => Color::Green,
        State::Skipped => Color::DarkGray,
        State::Error => Color::Red,
    }
}

fn fmt_duration(d: Duration) -> String {
    let secs = d.as_secs_f64();

    if secs >= 1.0 {
        format!("{secs:.1}s")
    } else {
        format!("{}ms", d.as_millis())
    }
}
