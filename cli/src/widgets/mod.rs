pub mod board;
pub mod diagnostic;
pub mod error;

use std::io::{IsTerminal, Write};

use ratatui::backend::IntoCrossterm;
use ratatui::buffer::Buffer;
use ratatui::crossterm::QueueableCommand;
use ratatui::crossterm::cursor::{MoveToColumn, MoveUp};
use ratatui::crossterm::style::{ContentStyle, PrintStyledContent};
use ratatui::crossterm::terminal::{Clear, ClearType};
use ratatui::layout::Rect;
use ratatui::style::Style;

/// Render a widget and print it to stderr as inline output — no alternate
/// screen, no event loop — so it composes with normal shell scrollback.
///
/// The widget is drawn into a standalone [`Buffer`] (its own [`Widget::render`]),
/// then the buffer is written out. On a TTY, each cell keeps its color and
/// modifiers via crossterm; when stderr is redirected or piped, styling is
/// dropped so logs and captures stay free of ANSI escapes.
pub fn print<W: ratatui::widgets::Widget>(widget: W, width: u16, height: u16) {
    let stderr = std::io::stderr();
    let is_terminal = stderr.is_terminal();
    render(widget, width, height, &mut stderr.lock(), is_terminal);
}

/// Like [`print`], but writes to stdout — for normal program output such as
/// diagnostics, keeping stderr reserved for errors.
pub fn println<W: ratatui::widgets::Widget>(widget: W, width: u16, height: u16) {
    let stdout = std::io::stdout();
    let is_terminal = stdout.is_terminal();
    render(widget, width, height, &mut stdout.lock(), is_terminal);
}

pub struct Painter {
    is_terminal: bool,
    prev_lines: u16,
}

impl Default for Painter {
    fn default() -> Self {
        Self::new()
    }
}

impl Painter {
    pub fn new() -> Self {
        Self {
            is_terminal: std::io::stdout().is_terminal(),
            prev_lines: 0,
        }
    }

    pub fn draw<W: ratatui::widgets::Widget>(&mut self, widget: W, width: u16, height: u16) {
        if !self.is_terminal {
            return;
        }

        let stdout = std::io::stdout();
        let mut out = stdout.lock();

        if self.prev_lines > 0 {
            let _ = out.queue(MoveUp(self.prev_lines));
            let _ = out.queue(MoveToColumn(0));
            let _ = out.queue(Clear(ClearType::FromCursorDown));
        }

        render(widget, width, height, &mut out, true);
        self.prev_lines = height;
    }

    pub fn finish<W: ratatui::widgets::Widget>(&mut self, widget: W, width: u16, height: u16) {
        let stdout = std::io::stdout();
        let is_terminal = stdout.is_terminal();
        let mut out = stdout.lock();

        if is_terminal && self.prev_lines > 0 {
            let _ = out.queue(MoveUp(self.prev_lines));
            let _ = out.queue(MoveToColumn(0));
            let _ = out.queue(Clear(ClearType::FromCursorDown));
        }

        render(widget, width, height, &mut out, is_terminal);
        self.prev_lines = 0;
    }
}

fn render<W: ratatui::widgets::Widget>(widget: W, width: u16, height: u16, out: &mut impl Write, color: bool) {
    let area = Rect::new(0, 0, width.max(1), height.max(1));
    let mut buf = Buffer::empty(area);
    widget.render(area, &mut buf);
    blit(&buf, out, color);
}

fn blit(buf: &Buffer, out: &mut impl Write, color: bool) {
    let area = buf.area;

    for y in area.top()..area.bottom() {
        // trailing empty cells add nothing but width; find the last non-blank
        let last = (area.left()..area.right())
            .rev()
            .find(|&x| buf[(x, y)].symbol() != " ")
            .map(|x| x + 1)
            .unwrap_or(area.left());

        // Coalesce runs of same-styled cells so each run emits one escape
        // sequence instead of one per character.
        let mut run = String::new();
        let mut run_style: Option<Style> = None;

        for x in area.left()..last {
            let cell = &buf[(x, y)];
            let style = Style::default().fg(cell.fg).bg(cell.bg).add_modifier(cell.modifier);

            if run_style != Some(style) {
                flush_run(out, &mut run, run_style, color);
                run_style = Some(style);
            }

            run.push_str(cell.symbol());
        }

        flush_run(out, &mut run, run_style, color);
        let _ = writeln!(out);
    }

    let _ = out.flush();
}

fn flush_run(out: &mut impl Write, run: &mut String, style: Option<Style>, color: bool) {
    if run.is_empty() {
        return;
    }

    if color {
        let style: ContentStyle = style.unwrap_or_default().into_crossterm();
        let _ = out.queue(PrintStyledContent(style.apply(run.as_str())));
    } else {
        let _ = write!(out, "{run}");
    }

    run.clear();
}
