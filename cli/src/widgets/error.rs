use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::Error;

/// A readable, colored error report for the terminal.
///
/// Built with a builder pattern on itself and rendered through ratatui's
/// [`Widget`] trait. Layout:
///
/// ```text
/// error: <title>
///   ┌ <source>
///   │ at: <path>
///   ╵ <message>
/// ```
///
/// `source` and `path` are optional and omitted when absent.
#[derive(Debug, Clone, Default)]
pub struct ErrorWidget {
    title: String,
    source: Option<String>,
    path: Option<String>,
    message: String,
}

impl ErrorWidget {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Self::default()
        }
    }

    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn height(&self) -> u16 {
        self.lines().len() as u16
    }

    pub fn width(&self) -> u16 {
        self.lines().iter().map(|line| line.width() as u16).max().unwrap_or(0)
    }

    pub fn lines(&self) -> Vec<Line<'static>> {
        let dim = Style::default().add_modifier(Modifier::DIM);
        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("error", Style::default().fg(Color::Red).bold()),
            Span::styled(": ", Style::default().fg(Color::Red).bold()),
            Span::raw(self.title.clone()),
        ]));

        if let Some(source) = &self.source {
            lines.push(Line::from(vec![
                Span::styled("  ┌ ", dim),
                Span::styled(source.clone(), Style::default().fg(Color::Cyan)),
            ]));
        }

        if let Some(path) = &self.path {
            lines.push(Line::from(vec![
                Span::styled("  │ ", dim),
                Span::styled("at: ", dim),
                Span::styled(path.clone(), Style::default().fg(Color::Yellow)),
            ]));
        }

        lines.push(Line::from(vec![Span::styled("  ╵ ", dim), Span::raw(self.message.clone())]));
        lines
    }
}

impl Widget for &ErrorWidget {
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

impl From<&Error> for ErrorWidget {
    fn from(err: &Error) -> Self {
        match err {
            Error::Glob(e) => e.into(),
            Error::Pattern(e) => e.into(),
            Error::Figment(e) => e.into(),
            Error::Clap(e) => Self::new("invalid arguments").message(e.to_string()),
            Error::NotFound(patterns) => Self::new("no files matched")
                .path(patterns.join(", "))
                .message("check the path, and quote glob patterns so the shell doesn't expand them"),
        }
    }
}

impl From<&figment::Error> for ErrorWidget {
    fn from(err: &figment::Error) -> Self {
        let source = err.metadata.as_ref().and_then(|m| m.source.as_ref()).map(|s| s.to_string());
        let path = (!err.path.is_empty()).then(|| err.path.join("."));
        let mut widget = Self::new("failed to load manifest").message(err.kind.to_string());

        if let Some(source) = source {
            widget = widget.source(source);
        }

        if let Some(path) = path {
            widget = widget.path(path);
        }

        widget
    }
}

impl From<&glob::GlobError> for ErrorWidget {
    fn from(err: &glob::GlobError) -> Self {
        Self::new("failed to read file")
            .source(err.path().display().to_string())
            .message(err.error().to_string())
    }
}

impl From<&glob::PatternError> for ErrorWidget {
    fn from(err: &glob::PatternError) -> Self {
        Self::new("invalid glob pattern")
            .path(format!("position {}", err.pos))
            .message(err.msg)
    }
}
