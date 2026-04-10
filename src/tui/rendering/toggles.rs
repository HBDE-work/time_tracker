use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::tui::smartcard::ReaderProbe;

/// Toggle indicators for the commands panel
pub(crate) fn render_toggles_column(
    smartcard_active: bool,
    reader_status: ReaderProbe,
    task_editor_open: bool,
) -> Vec<Line<'static>> {
    let mut content: Vec<Line<'static>> = Vec::new();
    content.push(Line::raw(""));

    let f1_key_style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let f1_label_style = Style::new().fg(Color::White);
    let (f1_state_label, f1_state_color) = if task_editor_open {
        ("ON", Color::Green)
    } else {
        ("OFF", Color::DarkGray)
    };
    content.push(Line::from(vec![
        Span::styled(
            f1_state_label,
            Style::new().fg(f1_state_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Edit Tasks ", f1_label_style),
        Span::styled("[F1]  ", f1_key_style),
    ]));

    let (f2_key_style, f2_label_style, f2_state_label, f2_state_color) = match reader_status {
        ReaderProbe::Unavailable => (
            Style::new().fg(Color::DarkGray),
            Style::new().fg(Color::DarkGray),
            "N/A",
            Color::DarkGray,
        ),
        ReaderProbe::NoReaders => (
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            Style::new().fg(Color::DarkGray),
            "OFF",
            Color::DarkGray,
        ),
        ReaderProbe::Available if smartcard_active => (
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            Style::new().fg(Color::White),
            "ON",
            Color::Green,
        ),
        ReaderProbe::Available => (
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            Style::new().fg(Color::White),
            "OFF",
            Color::DarkGray,
        ),
    };

    content.push(Line::from(vec![
        Span::styled(
            f2_state_label,
            Style::new().fg(f2_state_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Smartcard ", f2_label_style),
        Span::styled("[F2]  ", f2_key_style),
    ]));

    content
}
