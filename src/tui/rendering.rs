use chrono::Local;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::data::EventKind;
use crate::tracking_logic::calculate_worked;
use crate::tracking_logic::format_duration;
use crate::tracking_logic::last_event_kind;
use crate::tracking_logic::today_record;

/// Produces the text content for the upper "commands" panel
pub(crate) fn render_command_panel(feedback: &str) -> Vec<Line<'_>> {
    let actions: [(&str, char, &EventKind); 3] = [
        ("Go", 'g', &EventKind::Go),
        ("Pause", 'p', &EventKind::Pause),
        ("Stop", 's', &EventKind::Stop),
    ];

    let mut content: Vec<Line<'_>> = Vec::with_capacity(actions.len() + 3);
    content.push(Line::raw(""));

    for (name, hotkey, kind) in &actions {
        content.push(Line::from(vec![
            Span::styled(
                format!("  [{hotkey}] "),
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                *name,
                Style::new().fg(kind.color()).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    content.push(Line::raw(""));
    content.push(Line::from(Span::styled(
        format!("  {feedback}"),
        Style::new().fg(Color::White).add_modifier(Modifier::ITALIC),
    )));
    content
}

/// Produces the text content for the lower "status" panel
pub(crate) fn render_status_panel() -> Vec<Line<'static>> {
    let record = today_record();
    let actively_running = last_event_kind(&record) == Some(&EventKind::Go);
    let worked = calculate_worked(&record, actively_running);

    let mut content: Vec<Line<'static>> = Vec::new();

    content.push(Line::from(Span::styled(
        format!("── {} ──", record.date.format("%A, %Y-%m-%d")),
        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    content.push(Line::raw(""));

    for entry in &record.events {
        content.push(Line::from(vec![
            Span::styled(
                format!("  {} ", entry.time.format("%H:%M")),
                Style::new().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{}", entry.kind),
                Style::new().fg(entry.kind.color()),
            ),
        ]));
    }

    if actively_running {
        content.push(Line::raw(""));
        content.push(Line::from(Span::styled(
            "  ⏳ currently tracking",
            Style::new().fg(Color::Green).add_modifier(Modifier::ITALIC),
        )));
    }

    content.push(Line::raw(""));
    content.push(Line::from(vec![
        Span::styled("  Total: ", Style::new().add_modifier(Modifier::BOLD)),
        Span::styled(
            format_duration(worked),
            Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
    ]));
    content.push(Line::raw(""));
    content.push(Line::from(Span::styled(
        format!("  Last refresh: {}", Local::now().format("%H:%M:%S")),
        Style::new().fg(Color::DarkGray),
    )));

    content
}
