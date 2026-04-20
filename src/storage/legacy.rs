use chrono::NaiveDate;
use chrono::NaiveTime;
use serde::Deserialize;
use std::fs;

use crate::data::DayRecord;
use crate::data::Event;
use crate::data::EventKind;
use crate::data::Session;
use crate::data::TaskEvent;

use super::configuration::data_dir;
use super::fileio::save_record;

// Migrates all legacy JSON day-records in the data directory to TOML
//
// Each successfully migrated file is saved as .toml and the .json is removed
//
// Silently skips files that fail to parse
pub(super) fn migrate_all() {
    let dir = data_dir();
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let is_json = path.extension().is_some_and(|ext| ext == "json");

        if !is_json {
            continue;
        }

        let json_content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => continue,
        };

        let Some(migrated) = migrate_json(&json_content) else {
            continue;
        };

        save_record(&migrated);
        let _ = fs::remove_file(&path);
    }
}

#[derive(Deserialize)]
struct LegacyDayRecord {
    date: NaiveDate,
    events: Vec<LegacyEvent>,
    #[serde(default)]
    task_events: Vec<LegacyTaskEvent>,
}

#[derive(Deserialize)]
struct LegacyEvent {
    kind: EventKind,
    time: NaiveTime,
}

#[derive(Deserialize)]
struct LegacyTaskEvent {
    task: String,
    start: NaiveTime,
    end: Option<NaiveTime>,
}

// Converts legacy JSON content into the new session-based DayRecord
//
// The old format stored a flat list of events for the whole day
// We split them at each Stop boundary into separate sessions
pub(super) fn migrate_json(json_content: &str) -> Option<DayRecord> {
    let legacy: LegacyDayRecord = serde_json::from_str(json_content).ok()?;

    let sessions = split_into_sessions(&legacy.events, &legacy.task_events);

    Some(DayRecord {
        date: legacy.date,
        sessions,
    })
}

// Walks the flat event list and splits at Stop boundaries
//
// Each Go..Stop range (plus intermediate Pause/Go) becomes one Session
//
// A trailing open range (no final Stop) also becomes a session
fn split_into_sessions(events: &[LegacyEvent], task_events: &[LegacyTaskEvent]) -> Vec<Session> {
    let mut sessions: Vec<Session> = Vec::new();
    let mut current_events: Vec<Event> = Vec::new();
    let mut session_index: u32 = 0;

    for legacy_event in events {
        let event = Event {
            kind: legacy_event.kind.clone(),
            time: legacy_event.time,
        };

        current_events.push(event);

        if legacy_event.kind == EventKind::Stop {
            let session_start = current_events.first().map(|e| e.time);
            let session_end = current_events.last().map(|e| e.time);

            let matching_tasks = collect_tasks_for_range(task_events, session_start, session_end);

            sessions.push(Session {
                index: session_index,
                events: std::mem::take(&mut current_events),
                task_events: matching_tasks,
            });
            session_index += 1;
        }
    }

    // Remaining events form an open (non-stopped) session
    if !current_events.is_empty() {
        let session_start = current_events.first().map(|e| e.time);

        let matching_tasks = collect_tasks_for_range(task_events, session_start, None);

        sessions.push(Session {
            index: session_index,
            events: current_events,
            task_events: matching_tasks,
        });
    }

    sessions
}

// Assigns task events to a session based on time overlap
//
// A task belongs to a session if it started at or after `from`
// and (if `until` is set) started before the session ended
fn collect_tasks_for_range(
    task_events: &[LegacyTaskEvent],
    from: Option<NaiveTime>,
    until: Option<NaiveTime>,
) -> Vec<TaskEvent> {
    let Some(start_bound) = from else {
        return Vec::new();
    };

    task_events
        .iter()
        .filter(|te| te.start >= start_bound && until.is_none_or(|end| te.start <= end))
        .map(|te| TaskEvent {
            task: te.task.clone(),
            start: te.start,
            end: te.end,
        })
        .collect()
}
