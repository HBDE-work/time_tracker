use chrono::Local;
use chrono::NaiveDate;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::data::DayRecord;
use crate::data::glyphs::TUI;
use crate::storage::load_record;
use crate::tracking_logic::calculate_session_paused;
use crate::tracking_logic::calculate_session_total;
use crate::tracking_logic::calculate_task_durations;
use crate::tracking_logic::calculate_total_paused;
use crate::tracking_logic::calculate_total_time;
use crate::tracking_logic::calculate_worked;
use crate::tracking_logic::format_duration;
use crate::tracking_logic::format_duration_decimal;

pub(crate) fn render_status_panel(
    decimal_format: bool,
    date: Option<NaiveDate>,
) -> Vec<Line<'static>> {
    let today = Local::now().date_naive();
    let viewed_date = date.unwrap_or(today);
    let is_today = viewed_date == today;

    let record = load_record(viewed_date).unwrap_or_else(|| DayRecord::new(viewed_date));
    let actively_running = is_today && record.has_active_session();

    let total_time = calculate_total_time(&record, actively_running);
    let total_paused = calculate_total_paused(&record, actively_running);
    let total_worked = calculate_worked(&record, actively_running);

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

    let mut headline_spans = vec![Span::styled(
        format!(
            " {} {} {} ",
            TUI.horizontal_rule,
            record.date.format("%A, %Y-%m-%d"),
            TUI.horizontal_rule
        ),
        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )];
    headline_spans.extend(state_spans);

    content.push(Line::from(headline_spans));

    // Display detailed time breakdown
    let format_fn = if decimal_format {
        format_duration_decimal
    } else {
        format_duration
    };

    // Build time summary line with optional remaining time
    let mut time_summary_spans = vec![
        Span::styled("  Total: ", Style::new().add_modifier(Modifier::BOLD)),
        Span::styled(
            format_fn(total_time),
            Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  |  Paused: ", Style::new().add_modifier(Modifier::BOLD)),
        Span::styled(format_fn(total_paused), Style::new().fg(Color::Yellow)),
        Span::styled("  |  Worked: ", Style::new().add_modifier(Modifier::BOLD)),
        Span::styled(
            format_fn(total_worked),
            Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
    ];

    // Add remaining time if max hours is configured
    let config = crate::storage::TrackerConfig::load();
    if let Some(max_hours) = config.max_hours_per_day() {
        let max_duration = chrono::Duration::seconds((max_hours * 3600.0) as i64);
        let remaining = max_duration - total_worked;

        let (remaining_text, remaining_color) = if remaining.num_seconds() <= 0 {
            ("EXCEEDED".to_string(), Color::Red)
        } else if remaining.num_minutes() <= 30 {
            (format_fn(remaining), Color::Yellow)
        } else {
            (format_fn(remaining), Color::Green)
        };

        time_summary_spans.push(Span::styled(
            "  |  Remaining: ",
            Style::new().add_modifier(Modifier::BOLD),
        ));
        time_summary_spans.push(Span::styled(
            remaining_text,
            Style::new()
                .fg(remaining_color)
                .add_modifier(Modifier::BOLD),
        ));
    }

    content.push(Line::from(time_summary_spans));

    content.push(Line::raw(""));
    render_task_durations(&record, total_worked, decimal_format, &mut content);

    content.push(Line::raw(""));
    content.push(Line::from(Span::styled(
        format!("  Last refresh: {}", Local::now().format("%H:%M:%S")),
        Style::new().fg(Color::DarkGray),
    )));

    content.push(Line::raw(""));

    render_session_events(&record, actively_running, decimal_format, &mut content);

    content
}

pub(crate) fn render_task_durations(
    record: &DayRecord,
    worked: chrono::Duration,
    decimal_format: bool,
    content: &mut Vec<Line<'static>>,
) {
    let task_durations = calculate_task_durations(record);
    if task_durations.is_empty() {
        return;
    }

    let format_fn = if decimal_format {
        format_duration_decimal
    } else {
        format_duration
    };

    let config = crate::storage::TrackerConfig::load();

    let active_name = crate::tracking_logic::active_task_name(record);
    for (name, duration) in &task_durations {
        let is_running = active_name == Some(name.as_str());
        let style = if is_running {
            Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Color::White)
        };
        let marker = if is_running {
            format!("{} ", TUI.task_playing)
        } else {
            String::new()
        };

        let mut task_line = format!("  {marker}{name}: {}", format_fn(*duration));

        // Find task-specific max_hours by searching through configured task slots
        let task_max_hours = (0u8..=9)
            .filter_map(|slot| config.task_info(slot))
            .find(|info| info.name == *name)
            .and_then(|info| info.max_hours);

        // Add remaining time for this task if max hours is configured
        if let Some(max_hours) = task_max_hours {
            let max_duration = chrono::Duration::seconds((max_hours * 3600.0) as i64);
            let remaining = max_duration - *duration;

            if remaining.num_seconds() <= 0 {
                task_line.push_str(" (EXCEEDED)");
            } else {
                task_line.push_str(&format!(" (Remaining: {})", format_fn(remaining)));
            }
        }

        content.push(Line::from(Span::styled(task_line, style)));
    }

    let task_total: chrono::Duration = task_durations.iter().map(|(_, d)| *d).sum();
    let unassigned = worked - task_total;
    if unassigned.num_seconds() > 0 {
        content.push(Line::from(Span::styled(
            format!("  Unassigned: {}", format_fn(unassigned)),
            Style::new().fg(Color::DarkGray),
        )));
    }
}

pub(crate) fn render_session_events(
    record: &DayRecord,
    actively_running: bool,
    decimal_format: bool,
    content: &mut Vec<Line<'static>>,
) {
    let session_count = record.sessions.len();

    let format_fn = if decimal_format {
        format_duration_decimal
    } else {
        format_duration
    };

    for (idx, session) in record.sessions.iter().enumerate().rev() {
        let is_last = idx + 1 == session_count;
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

        // Calculate and display session metrics
        let session_total = calculate_session_total(session, actively_running && is_last);
        let session_paused = calculate_session_paused(session, actively_running && is_last);
        let session_worked = session_total - session_paused;

        content.push(Line::from(vec![
            Span::styled("    ", Style::new()),
            Span::styled("Total: ", Style::new().fg(Color::White)),
            Span::styled(format_fn(session_total), Style::new().fg(Color::White)),
            Span::styled("  |  Paused: ", Style::new().fg(Color::White)),
            Span::styled(format_fn(session_paused), Style::new().fg(Color::Yellow)),
            Span::styled("  |  Worked: ", Style::new().fg(Color::White)),
            Span::styled(format_fn(session_worked), Style::new().fg(Color::Green)),
        ]));

        content.push(Line::raw(""));
    }
}
