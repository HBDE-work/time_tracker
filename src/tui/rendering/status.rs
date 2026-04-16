use chrono::Local;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::data::DayRecord;
use crate::data::glyphs::TUI;
use crate::tracking_logic::calculate_task_durations;
use crate::tracking_logic::calculate_worked;
use crate::tracking_logic::format_duration;
use crate::tracking_logic::today_record;

pub(crate) fn render_status_panel() -> Vec<Line<'static>> {
    let record = today_record();
    let actively_running = record.has_active_session();
    let worked = calculate_worked(&record, actively_running);

    let mut content: Vec<Line<'static>> = Vec::new();

    let mut state_spans = Vec::new();
    if record.is_tracking() {
        state_spans.push(Span::styled(
            " | tracking",
            Style::new().fg(Color::Green).add_modifier(Modifier::ITALIC),
        ));
    }
    if record.is_paused() {
        state_spans.push(Span::styled(
            " | paused",
            Style::new()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC),
        ));
    }

    let mut headline_spans = vec![
        Span::styled(
            format!(
                " {} {} {} ",
                TUI.horizontal_rule,
                record.date.format("%A, %Y-%m-%d"),
                TUI.horizontal_rule
            ),
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled("| Total: ", Style::new().add_modifier(Modifier::BOLD)),
        Span::styled(
            format_duration(worked),
            Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
    ];
    headline_spans.extend(state_spans);

    content.push(Line::from(headline_spans));

    render_task_durations(&record, worked, &mut content);

    content.push(Line::raw(""));
    content.push(Line::from(Span::styled(
        format!("  Last refresh: {}", Local::now().format("%H:%M:%S")),
        Style::new().fg(Color::DarkGray),
    )));

    content.push(Line::raw(""));

    render_session_events(&record, &mut content);

    content
}

pub(crate) fn render_task_durations(
    record: &DayRecord,
    worked: chrono::Duration,
    content: &mut Vec<Line<'static>>,
) {
    let task_durations = calculate_task_durations(record);
    if task_durations.is_empty() {
        return;
    }

    let active_name = crate::tracking_logic::active_task_name(record);
    for (name, duration) in &task_durations {
        let is_running = active_name == Some(name.as_str());
        let style = if is_running {
            Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Color::White)
        };
        let marker = if is_running {
            format!(" {} ", TUI.task_playing)
        } else {
            String::new()
        };
        content.push(Line::from(Span::styled(
            format!("  {marker}{name}: {}", format_duration(*duration)),
            style,
        )));
    }

    let task_total: chrono::Duration = task_durations.iter().map(|(_, d)| *d).sum();
    let unassigned = worked - task_total;
    if unassigned.num_seconds() > 0 {
        content.push(Line::from(Span::styled(
            format!("  Unassigned: {}", format_duration(unassigned)),
            Style::new().fg(Color::DarkGray),
        )));
    }
}

pub(crate) fn render_session_events(record: &DayRecord, content: &mut Vec<Line<'static>>) {
    for session in record.sessions.iter().rev() {
        let (state_label, state_color) = if session.is_active() {
            ("tracking", Color::Green)
        } else if session.is_stopped() {
            ("stopped", Color::Red)
        } else {
            ("paused", Color::Yellow)
        };

        content.push(Line::from(vec![
            Span::styled(
                format!("  Session {} ", session.index),
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("({state_label})"), Style::new().fg(state_color)),
        ]));

        for entry in session.events.iter().rev() {
            content.push(Line::from(vec![
                Span::styled(
                    format!("    {} ", entry.time.format("%H:%M")),
                    Style::new().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{}", entry.kind),
                    Style::new().fg(entry.kind.color()),
                ),
            ]));
        }

        content.push(Line::raw(""));
    }
}
