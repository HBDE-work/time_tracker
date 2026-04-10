use chrono::{Duration, Local, NaiveTime};

use crate::data::glyphs::CLI;
use crate::data::{DayRecord, TaskEvent};
use crate::storage::save_record;

use super::actions::{format_duration, today_record};

pub(crate) fn start_task(task_name: &str) -> String {
    let mut record = today_record();
    let now = Local::now().time();

    let _prev = suspend_current_task(&mut record, now);

    let Some(session) = record.current_session_mut() else {
        return "No active session. Use 'go' first.".into();
    };

    session.task_events.push(TaskEvent {
        task: task_name.to_owned(),
        start: now,
        end: None,
    });
    save_record(&record);

    format!(
        "{} Task \"{}\" started at {}",
        CLI.task_playing,
        task_name,
        now.format("%H:%M"),
    )
}

pub(crate) fn stop_active_task() -> Option<String> {
    let mut record = today_record();
    let now = Local::now().time();
    let feedback = suspend_current_task(&mut record, now);

    if feedback.is_some() {
        save_record(&record);
    }
    feedback
}

pub(crate) fn active_task_name(record: &DayRecord) -> Option<&str> {
    let session = record.current_session()?;
    session
        .task_events
        .iter()
        .rev()
        .find(|te| te.end.is_none())
        .map(|te| te.task.as_str())
}

/// Aggregates per-task durations across all sessions
///
/// Open timespans count up to now
///
/// Returns tasks in the order they were first used (chronological)
pub(crate) fn calculate_task_durations(record: &DayRecord) -> Vec<(String, Duration)> {
    let now = Local::now().time();
    let mut totals: Vec<(String, Duration)> = Vec::new();

    for session in &record.sessions {
        for te in &session.task_events {
            let stop = te.end.unwrap_or(now);
            let elapsed = stop - te.start;

            if let Some(entry) = totals.iter_mut().find(|(name, _)| name == &te.task) {
                entry.1 += elapsed;
            } else {
                totals.push((te.task.clone(), elapsed));
            }
        }
    }

    totals
}

pub(crate) fn format_task_summary(record: &DayRecord, total_worked: Duration) -> String {
    let durations = calculate_task_durations(record);
    if durations.is_empty() {
        return String::new();
    }

    let current = active_task_name(record);
    let mut buf = String::from("\n  Tasks:\n");

    for (name, dur) in &durations {
        let running = current == Some(name.as_str());
        let marker = if running {
            format!(" {} ", CLI.task_playing)
        } else {
            String::new()
        };
        buf.push_str(&format!("    {marker}{name}: {}\n", format_duration(*dur)));
    }

    let task_total: Duration = durations.iter().map(|(_, d)| *d).sum();
    let unassigned = total_worked - task_total;
    if unassigned.num_seconds() > 0 {
        buf.push_str(&format!(
            "\n    Unassigned: {}\n",
            format_duration(unassigned),
        ));
    }

    buf
}

fn suspend_current_task(record: &mut DayRecord, at: NaiveTime) -> Option<String> {
    let session = record.current_session_mut()?;
    let open = session
        .task_events
        .iter_mut()
        .rev()
        .find(|te| te.end.is_none())?;

    open.end = Some(at);
    let elapsed = at - open.start;

    let msg = format!(
        "{} Task \"{}\" suspended ({})",
        CLI.task_stopped,
        open.task,
        format_duration(elapsed),
    );
    Some(msg)
}
