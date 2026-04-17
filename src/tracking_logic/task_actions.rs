use chrono::{Duration, Local, NaiveTime};

use crate::data::glyphs::CLI;
use crate::data::{DayRecord, TaskEvent};
use crate::storage::save_record;

use super::format_duration;
use super::today_record;

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
/// Open timespans count up to now for today, or end-of-day for past dates
pub(crate) fn calculate_task_durations(record: &DayRecord) -> Vec<(String, Duration)> {
    let now = Local::now();
    let is_today = record.date == now.date_naive();

    // For today, use current time; for past days, use end-of-day
    let default_end = if is_today {
        now.time()
    } else {
        NaiveTime::from_hms_opt(23, 59, 59).unwrap()
    };

    let mut totals: Vec<(String, Duration)> = Vec::new();

    for session in &record.sessions {
        for task_event in &session.task_events {
            let stop = task_event.end.unwrap_or(default_end);
            let elapsed = stop - task_event.start;

            if let Some(entry) = totals.iter_mut().find(|(name, _)| name == &task_event.task) {
                entry.1 += elapsed;
            } else {
                totals.push((task_event.task.clone(), elapsed));
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
    let mut summary = String::from("\n  Tasks:\n");

    for (name, duration) in &durations {
        let running = current == Some(name.as_str());
        let marker = if running {
            format!(" {} ", CLI.task_playing)
        } else {
            String::new()
        };
        summary.push_str(&format!(
            "    {marker}{name}: {}\n",
            format_duration(*duration)
        ));
    }

    let task_total: Duration = durations.iter().map(|(_, d)| *d).sum();
    let unassigned = total_worked - task_total;
    if unassigned.num_seconds() > 0 {
        summary.push_str(&format!(
            "\n    Unassigned: {}\n",
            format_duration(unassigned),
        ));
    }

    summary
}

fn suspend_current_task(record: &mut DayRecord, at: NaiveTime) -> Option<String> {
    let session = record.current_session_mut()?;
    let open = session
        .task_events
        .iter_mut()
        .rev()
        .find(|task_event| task_event.end.is_none())?;

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
