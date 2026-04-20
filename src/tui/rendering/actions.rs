use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::data::EventKind;

/// Action hotkeys for the commands panel
pub(crate) fn render_actions_column() -> Vec<Line<'static>> {
    let actions: [(&str, char, &EventKind); 3] = [
        ("Go", 'g', &EventKind::Go),
        ("Pause", 'p', &EventKind::Pause),
        ("Stop", 's', &EventKind::Stop),
    ];

    let mut content: Vec<Line<'static>> = Vec::with_capacity(actions.len() + 2);
    content.push(Line::raw(""));

    for (name, hotkey, kind) in &actions {
        content.push(Line::from(vec![
            Span::styled(
                format!("  [{hotkey}] "),
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                name.to_string(),
                Style::new().fg(kind.color()).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    // Edit action
    content.push(Line::from(vec![
        Span::styled(
            "  [e] ",
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Edit",
            Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
    ]));

    content
}
