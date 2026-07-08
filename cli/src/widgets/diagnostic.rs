use nova::{Diagnostic, Severity};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

pub fn new<'a>(diagnostic: &'a Diagnostic) -> Widget<'a> {
    Widget { diagnostic }
}

/// A readable, colored render of a [`Diagnostic`] tree for the terminal.
///
/// Built from a `&Diagnostic` and rendered through ratatui's [`Widget`] trait,
/// matching the inline style of [`super::ErrorWidget`]. Each node shows a
/// severity glyph and message, and children nest with tree gutters:
///
/// ```text
/// ● error  something failed
///   ├─ info  detail one
///   ╰─ warn  rate 90 exceeded
/// ```
///
/// A node's glyph color reflects its rolled-up severity (the max across itself
/// and its descendants), so a parent flagged by a deeper error reads as an error.
#[derive(Debug, Clone)]
pub struct Widget<'a> {
    diagnostic: &'a Diagnostic,
}

impl<'a> Widget<'a> {
    pub fn height(&self) -> u16 {
        self.lines().len() as u16
    }

    pub fn width(&self) -> u16 {
        self.lines().iter().map(|line| line.width() as u16).max().unwrap_or(0)
    }

    pub fn lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        self.push_node(self.diagnostic, String::new(), true, true, &mut lines);
        lines
    }

    fn push_node(&self, node: &Diagnostic, prefix: String, is_root: bool, is_last: bool, lines: &mut Vec<Line<'static>>) {
        let dim = Style::default().add_modifier(Modifier::DIM);
        let severity = node.severity();
        let mut spans = Vec::new();

        if !is_root {
            let branch = if is_last { "╰─ " } else { "├─ " };
            spans.push(Span::styled(prefix.clone(), dim));
            spans.push(Span::styled(branch, dim));
        }

        spans.push(Span::styled(glyph(severity), Style::default().fg(color(severity)).bold()));
        spans.push(Span::styled(
            format!(" {} ", severity),
            Style::default().fg(color(severity)).bold(),
        ));

        if let Some(message) = &node.message {
            spans.push(Span::raw(message.clone()));
        }

        lines.push(Line::from(spans));

        let child_prefix = if is_root {
            String::new()
        } else if is_last {
            format!("{prefix}   ")
        } else {
            format!("{prefix}│  ")
        };

        let last = node.children.len().saturating_sub(1);
        for (i, child) in node.children.iter().enumerate() {
            self.push_node(child, child_prefix.clone(), false, i == last, lines);
        }
    }
}

fn glyph(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "●",
        Severity::Warn => "▲",
        Severity::Error => "●",
    }
}

fn color(severity: Severity) -> Color {
    match severity {
        Severity::Info => Color::Cyan,
        Severity::Warn => Color::Yellow,
        Severity::Error => Color::Red,
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

impl<'a> From<&'a Diagnostic> for Widget<'a> {
    fn from(diagnostic: &'a Diagnostic) -> Self {
        new(diagnostic)
    }
}
