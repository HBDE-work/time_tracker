use chrono::Local;

use crate::data::Event;
use crate::data::EventKind;

use super::storage::{load_record, save_record};
use super::timer::calculate_worked;

pub(crate) fn format_duration(d: chrono::Duration) -> String {
    let total_minutes = d.num_minutes();
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{hours}h {minutes:02}m")
}

/// Loads today's record or creates an empty one
pub(crate) fn today_record() -> crate::data::DayRecord {
    let today = Local::now().date_naive();
    load_record(today).unwrap_or(crate::data::DayRecord {
        date: today,
        events: Vec::new(),
    })
}

/// Returns the kind of the last event in the record
pub(crate) fn last_event_kind(record: &crate::data::DayRecord) -> Option<&EventKind> {
    record.events.last().map(|e| &e.kind)
}

/// Executes a tracker action on today's record
/// Returns message describing what happened
pub(crate) fn execute_action(action: EventKind) -> String {
    let mut record = today_record();
    let now = Local::now().time();
    let previous = last_event_kind(&record).cloned();

    match (&action, previous) {
        // Go
        (EventKind::Go, Some(EventKind::Go)) => "Already tracking.".into(),
        (EventKind::Go, Some(EventKind::Stop)) => "Already stopped for today.".into(),
        (EventKind::Go, _) => {
            let first = record.events.is_empty();
            record.events.push(Event {
                kind: EventKind::Go,
                time: now,
            });
            save_record(&record);
            if first {
                format!("Started tracking at {}", now.format("%H:%M"))
            } else {
                format!("Resumed tracking at {}", now.format("%H:%M"))
            }
        }

        // Pause
        (EventKind::Pause, None) => "Not tracking yet. Use 'go' first.".into(),
        (EventKind::Pause, Some(EventKind::Pause)) => "Already paused.".into(),
        (EventKind::Pause, Some(EventKind::Stop)) => "Already stopped for today.".into(),
        (EventKind::Pause, _) => {
            record.events.push(Event {
                kind: EventKind::Pause,
                time: now,
            });
            save_record(&record);
            let worked = calculate_worked(&record, false);
            format!(
                "Paused at {}. Worked so far: {}",
                now.format("%H:%M"),
                format_duration(worked)
            )
        }

        // Stop
        (EventKind::Stop, None) => "Not tracking yet. Use 'go' first.".into(),
        (EventKind::Stop, Some(EventKind::Stop)) => "Already stopped for today.".into(),
        (EventKind::Stop, _) => {
            record.events.push(Event {
                kind: EventKind::Stop,
                time: now,
            });
            save_record(&record);
            let worked = calculate_worked(&record, false);
            format!(
                "Stopped at {}. Total today: {}",
                now.format("%H:%M"),
                format_duration(worked)
            )
        }
    }
}
