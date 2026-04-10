use chrono::Local;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::data::DayRecord;
use crate::data::EventKind;
use crate::data::glyphs::TUI;
use crate::storage::TrackerConfig;
use crate::tracking_logic::calculate_task_durations;
use crate::tracking_logic::calculate_worked;
use crate::tracking_logic::format_duration;
use crate::tracking_logic::today_record;

use super::smartcard::ReaderProbe;

/// Produces the left-hand content of the upper panel: action hotkeys
pub(crate) fn render_actions_column() -> Vec<Line<'static>> {
    let actions: [(&str, char, &EventKind); 3] = [
        ("Go", 'g', &EventKind::Go),
        ("Pause", 'p', &EventKind::Pause),
        ("Stop", 's', &EventKind::Stop),
    ];

    let mut content: Vec<Line<'static>> = Vec::with_capacity(actions.len() + 1);
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

    content
}

pub(crate) fn render_task_indicators(
    config: &TrackerConfig,
    active_task: Option<u8>,
) -> Vec<Line<'static>> {
    const COL_WIDTH: usize = 22;

    // Collect configured slots
    let slots: Vec<(u8, &str)> = (0u8..=9)
        .filter_map(|s| config.task_name(s).map(|n| (s, n)))
        .collect();

    let mut content: Vec<Line<'static>> = Vec::new();
    content.push(Line::raw(""));

    for row in slots.chunks(3) {
        let mut timespans: Vec<Span<'static>> = Vec::new();

        for (slot, name) in row {
            let is_active = active_task == Some(*slot);

            // Build the cell text and truncate/pad to COL_WIDTH
            let (prefix, display_name) = if is_active {
                (format!("[{slot}]{}", TUI.task_playing), *name)
            } else {
                (format!("[{slot}] "), *name)
            };

            // Truncate name if it would exceed the column width
            let prefix_len = prefix.chars().count();
            let max_name = COL_WIDTH.saturating_sub(prefix_len);
            let truncated: String = display_name.chars().take(max_name).collect();

            let style = if is_active {
                Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(Color::White)
            };

            // Render the [N] part in its own color, then the name
            if is_active {
                timespans.push(Span::styled(
                    format!("[{slot}]"),
                    Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
                ));
                let play = TUI.task_playing;
                let padded_name = format!("{play}{truncated:<pad$}", pad = max_name);
                timespans.push(Span::styled(padded_name, style));
            } else {
                timespans.push(Span::styled(
                    format!("[{slot}]"),
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ));
                let padded_name = format!(" {truncated:<pad$}", pad = max_name);
                timespans.push(Span::styled(padded_name, style));
            }
        }

        content.push(Line::from(timespans));
    }

    content
}

/// Produces the right-hand content of the upper panel: F-key toggle indicators
///
/// Each toggle is rendered as `[F<n>] Label  ON/OFF`
/// and greyed out when the feature is unavailable
pub(crate) fn render_toggles_column(
    smartcard_active: bool,
    reader_status: ReaderProbe,
    task_editor_open: bool,
) -> Vec<Line<'static>> {
    let mut content: Vec<Line<'static>> = Vec::new();
    content.push(Line::raw(""));

    // F1: Task Editor
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

    // F2: Smartcard auto-tracking
    //
    //  Unavailable (library missing)  => fully greyed out, "N/A"
    //  NoReaders   (lib OK, no reader)=> key cyan (re-probe on press), "OFF" dimmed
    //  Available + off                => normal "OFF"
    //  Available + on                 => "ON" in green
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

    // more toggles...

    content
}

/// Single-line feedback styled as italic white text
pub(crate) fn render_feedback_line(feedback: &str) -> Line<'_> {
    Line::from(Span::styled(
        format!(" {feedback}"),
        Style::new().fg(Color::White).add_modifier(Modifier::ITALIC),
    ))
}

pub(crate) fn render_status_panel() -> Vec<Line<'static>> {
    let record = today_record();
    let actively_running = record.has_active_session();
    let worked = calculate_worked(&record, actively_running);

    let mut content: Vec<Line<'static>> = Vec::new();

    content.push(Line::from(vec![
        Span::styled(
            format!(
                "{} {} ",
                TUI.horizontal_rule,
                record.date.format("%A, %Y-%m-%d")
            ),
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Total: ", Style::new().add_modifier(Modifier::BOLD)),
        Span::styled(
            format_duration(worked),
            Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            if actively_running { "  tracking" } else { "" },
            Style::new().fg(Color::Green).add_modifier(Modifier::ITALIC),
        ),
    ]));

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

fn render_task_durations(
    record: &DayRecord,
    worked: chrono::Duration,
    content: &mut Vec<Line<'static>>,
) {
    let task_durations = calculate_task_durations(record);
    if task_durations.is_empty() {
        return;
    }

    let active_name = crate::tracking_logic::active_task_name(record);
    for (name, dur) in &task_durations {
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
            format!("  {marker}{name}: {}", format_duration(*dur)),
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

fn render_session_events(record: &DayRecord, content: &mut Vec<Line<'static>>) {
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

pub(crate) fn render_task_editor_panel(
    config: &TrackerConfig,
    editing_slot: Option<u8>,
    editing_buffer: &str,
) -> Vec<Line<'static>> {
    let mut content: Vec<Line<'static>> = Vec::new();

    let rule = TUI.horizontal_rule;
    content.push(Line::from(Span::styled(
        format!(
            "{rule} Task Editor  (press a number to edit, Enter to save, Esc to cancel) {rule}"
        ),
        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    content.push(Line::raw(""));

    for slot in 0u8..=9 {
        let is_editing = editing_slot == Some(slot);

        if is_editing {
            // Show the live editing buffer with a block cursor
            content.push(Line::from(vec![
                Span::styled(
                    format!("  [{slot}] "),
                    Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    editing_buffer.to_owned(),
                    Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    TUI.cursor_block,
                    Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]));
        } else {
            let name = config.task_name(slot).unwrap_or("");
            let name_style = if name.is_empty() {
                Style::new().fg(Color::DarkGray)
            } else {
                Style::new().fg(Color::White)
            };
            let display = if name.is_empty() {
                TUI.empty_slot.to_owned()
            } else {
                name.to_owned()
            };
            content.push(Line::from(vec![
                Span::styled(
                    format!("  [{slot}] "),
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled(display, name_style),
            ]));
        }
    }

    content.push(Line::raw(""));
    content.push(Line::from(Span::styled(
        "  Press F1 or Esc to close editor",
        Style::new()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    )));

    content
}
