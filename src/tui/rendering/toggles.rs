use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::data::glyphs::TUI;
use crate::tui::smartcard::ReaderProbe;

/// Toggle indicators for the commands panel
pub(crate) fn render_toggles_column(
    smartcard_active: bool,
    reader_status: ReaderProbe,
    task_editor_open: bool,
    decimal_time_format: bool,
    history_mode: bool,
) -> Vec<Line<'static>> {
    let mut content: Vec<Line<'static>> = Vec::new();
    content.push(Line::raw(""));

    // F1 - Task Editor
    let (f1_state, f1_color) = if task_editor_open {
        ("ON ", Color::Green)
    } else {
        ("OFF", Color::DarkGray)
    };
    content.push(Line::from(vec![
        Span::styled(
            "[F1]",
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Tasks ", Style::new().fg(Color::White)),
        Span::styled(
            f1_state,
            Style::new().fg(f1_color).add_modifier(Modifier::BOLD),
        ),
    ]));

    // F2 - Smartcard
    let (f2_key_color, f2_label_color, f2_state, f2_state_color) = match reader_status {
        ReaderProbe::Unavailable => (Color::DarkGray, Color::DarkGray, "N/A", Color::DarkGray),
        ReaderProbe::NoReaders => (Color::Cyan, Color::DarkGray, "OFF", Color::DarkGray),
        ReaderProbe::Available if smartcard_active => {
            (Color::Cyan, Color::White, "ON ", Color::Green)
        }
        ReaderProbe::Available => (Color::Cyan, Color::White, "OFF", Color::DarkGray),
    };

    content.push(Line::from(vec![
        Span::styled(
            "[F2]",
            Style::new().fg(f2_key_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Card  ", Style::new().fg(f2_label_color)),
        Span::styled(
            f2_state,
            Style::new().fg(f2_state_color).add_modifier(Modifier::BOLD),
        ),
    ]));

    // F3 - History
    let (f3_state, f3_color) = if history_mode {
        ("ON ", Color::Green)
    } else {
        ("OFF", Color::DarkGray)
    };
    content.push(Line::from(vec![
        Span::styled(
            "[F3]",
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("  {}    ", TUI.book), Style::new().fg(Color::White)),
        Span::styled(
            f3_state,
            Style::new().fg(f3_color).add_modifier(Modifier::BOLD),
        ),
    ]));

    // F12 - Time Format
    let (f12_state, f12_color) = if decimal_time_format {
        ("DEC", Color::Yellow)
    } else {
        ("HMS", Color::Cyan)
    };
    content.push(Line::from(vec![
        Span::styled(
            "[F12]",
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Unit  ", Style::new().fg(Color::White)),
        Span::styled(
            f12_state,
            Style::new().fg(f12_color).add_modifier(Modifier::BOLD),
        ),
    ]));

    content
}
