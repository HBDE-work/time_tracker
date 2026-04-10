use chrono::{Duration, Local, NaiveTime};

use crate::data::{DayRecord, TaskEvent};
use crate::storage::save_record;

use super::actions::{format_duration, today_record};

/// Begin tracking a named task
///
/// Any previously-open task timespan is closed first so the record stays consistent
///
/// Returns feedback string for the TUI / CLI
pub(crate) fn start_task(task_name: &str) -> String {
    let mut record = today_record();
    let now = Local::now().time();

    // Close a running task, if
    let _prev = suspend_current_task(&mut record, now);

    let timespan = TaskEvent {
        task: task_name.to_owned(),
        start: now,
        end: None,
    };
    record.task_events.push(timespan);
    save_record(&record);

    format!(
        "▶ Task \"{}\" started at {}",
        task_name,
        now.format("%H:%M")
    )
}

/// Stop whichever task is currently open and return to global time
///
/// Returns `None` when no task was open
pub(crate) fn stop_active_task() -> Option<String> {
    let mut record = today_record();
    let now = Local::now().time();
    let feedback = suspend_current_task(&mut record, now);

    if feedback.is_some() {
        save_record(&record);
    }
    feedback
}

/// Inspect the current record and return the `name` of the running task | `None` when tracking global time
pub(crate) fn active_task_name(record: &DayRecord) -> Option<&str> {
    record
        .task_events
        .iter()
        .rev()
        .find(|te| te.end.is_none())
        .map(|te| te.task.as_str())
}

/// Aggregate per-task durations for today
///
/// Open timespans (still running) count up to `now`
///
/// Returns a vec of `(task_name, total_duration)` in the order each task
/// was first used today (chronological)
pub(crate) fn calculate_task_durations(record: &DayRecord) -> Vec<(String, Duration)> {
    let now = Local::now().time();
    let mut totals: Vec<(String, Duration)> = Vec::new();

    for te in &record.task_events {
        let stop = te.end.unwrap_or(now);
        let elapsed = stop - te.start;

        if let Some(entry) = totals.iter_mut().find(|(name, _)| name == &te.task) {
            entry.1 += elapsed;
        } else {
            totals.push((te.task.clone(), elapsed));
        }
    }

    totals
}

/// Produce a multi-line summary suitable for `tracker status` CLI output
///
/// `total_worked` is the overall tracked time so we can show unassigned time
///
/// Returns an empty string when there are no task events
pub(crate) fn format_task_summary(record: &DayRecord, total_worked: Duration) -> String {
    let durations = calculate_task_durations(record);
    if durations.is_empty() {
        return String::new();
    }

    let current = active_task_name(record);
    let mut buf = String::from("\n  Tasks:\n");

    for (name, dur) in &durations {
        let running = current == Some(name.as_str());
        let marker = if running { " ▶ " } else { "" };
        buf.push_str(&format!("    {marker}{name}: {}\n", format_duration(*dur)));
    }

    let task_total: Duration = durations.iter().map(|(_, d)| *d).sum();
    let unassigned = total_worked - task_total;
    if unassigned.num_seconds() > 0 {
        buf.push_str(&format!(
            "\n    Unassigned: {}\n",
            format_duration(unassigned)
        ));
    }

    buf
}

/// Close the most-recent open task timespan by writing `end = at`
///
/// Returns a feedback message describing what was closed, or `None` if nothing was open
fn suspend_current_task(record: &mut DayRecord, at: NaiveTime) -> Option<String> {
    // Walk backwards to find the first timespan with no `end`
    let open = record
        .task_events
        .iter_mut()
        .rev()
        .find(|te| te.end.is_none())?;

    open.end = Some(at);
    let elapsed = at - open.start;

    let msg = format!(
        "⏹ Task \"{}\" suspended ({})",
        open.task,
        format_duration(elapsed),
    );
    Some(msg)
}
