mod error;

use std::io::{IsTerminal, Write};

pub use error::*;
use ratatui::backend::IntoCrossterm;
use ratatui::buffer::Buffer;
use ratatui::crossterm::QueueableCommand;
use ratatui::crossterm::style::{ContentStyle, PrintStyledContent};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

/// Render a widget and print it to stderr as inline output — no alternate
/// screen, no event loop — so it composes with normal shell scrollback.
///
/// The widget is drawn into a standalone [`Buffer`] (its own [`Widget::render`]),
/// then the buffer is written out. On a TTY, each cell keeps its color and
/// modifiers via crossterm; when stderr is redirected or piped, styling is
/// dropped so logs and captures stay free of ANSI escapes.
pub fn print<W: Widget>(widget: W, width: u16, height: u16) {
    let area = Rect::new(0, 0, width.max(1), height.max(1));
    let mut buf = Buffer::empty(area);
    widget.render(area, &mut buf);
    blit(&buf);
}

fn blit(buf: &Buffer) {
    let mut stderr = std::io::stderr();
    let color = stderr.is_terminal();
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
                flush_run(&mut stderr, &mut run, run_style, color);
                run_style = Some(style);
            }

            run.push_str(cell.symbol());
        }

        flush_run(&mut stderr, &mut run, run_style, color);
        let _ = writeln!(stderr);
    }

    let _ = stderr.flush();
}

fn flush_run(stderr: &mut impl Write, run: &mut String, style: Option<Style>, color: bool) {
    if run.is_empty() {
        return;
    }

    if color {
        let style: ContentStyle = style.unwrap_or_default().into_crossterm();
        let _ = stderr.queue(PrintStyledContent(style.apply(run.as_str())));
    } else {
        let _ = write!(stderr, "{run}");
    }

    run.clear();
}
