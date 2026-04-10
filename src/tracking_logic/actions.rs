use chrono::Local;

use crate::data::DayRecord;
use crate::data::Event;
use crate::data::EventKind;
use crate::data::Session;
use crate::storage::load_record;
use crate::storage::save_record;

use super::timer::calculate_worked;

pub(crate) fn format_duration(d: chrono::Duration) -> String {
    let total_minutes = d.num_minutes();
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{hours}h {minutes:02}m")
}

pub(crate) fn today_record() -> DayRecord {
    let today = Local::now().date_naive();
    load_record(today).unwrap_or_else(|| DayRecord::new(today))
}

pub(crate) fn execute_action(action: EventKind) -> String {
    let mut record = today_record();
    let now = Local::now().time();

    match action {
        EventKind::Go => execute_go(&mut record, now),
        EventKind::Pause => execute_pause(&mut record, now),
        EventKind::Stop => execute_stop(&mut record, now),
    }
}

fn execute_go(record: &mut DayRecord, now: chrono::NaiveTime) -> String {
    let needs_new_session = match record.current_session() {
        None => true,
        Some(session) => session.is_stopped(),
    };

    if let Some(session) = record.current_session()
        && session.is_active()
    {
        return "Already tracking.".into();
    }

    if needs_new_session {
        let index = record.next_session_index();
        let mut session = Session::new(index);
        session.events.push(Event {
            kind: EventKind::Go,
            time: now,
        });
        record.sessions.push(session);
        save_record(record);

        if index == 0 {
            format!("Session 0: Started tracking at {}", now.format("%H:%M"))
        } else {
            format!("Session {index}: Started at {}", now.format("%H:%M"))
        }
    } else {
        // Resume within existing paused session
        let session = record.current_session_mut().unwrap();
        session.events.push(Event {
            kind: EventKind::Go,
            time: now,
        });
        let index = session.index;
        save_record(record);

        format!("Session {index}: Resumed at {}", now.format("%H:%M"))
    }
}

fn execute_pause(record: &mut DayRecord, now: chrono::NaiveTime) -> String {
    let Some(session) = record.current_session() else {
        return "Not tracking yet. Use 'go' first.".into();
    };

    if session.is_stopped() {
        return format!(
            "Session {} is already stopped. Use 'go' to start a new one.",
            session.index,
        );
    }

    if !session.is_active() {
        return "Already paused.".into();
    }

    let index = session.index;
    let session = record.current_session_mut().unwrap();
    session.events.push(Event {
        kind: EventKind::Pause,
        time: now,
    });
    save_record(record);

    let worked = calculate_worked(record, false);
    format!(
        "Session {index}: Paused at {}. Worked so far: {}",
        now.format("%H:%M"),
        format_duration(worked),
    )
}

fn execute_stop(record: &mut DayRecord, now: chrono::NaiveTime) -> String {
    let Some(session) = record.current_session() else {
        return "Not tracking yet. Use 'go' first.".into();
    };

    if session.is_stopped() {
        return format!(
            "Session {} is already stopped. Use 'go' to start a new one.",
            session.index,
        );
    }

    let index = session.index;
    let session = record.current_session_mut().unwrap();
    session.events.push(Event {
        kind: EventKind::Stop,
        time: now,
    });
    save_record(record);

    let worked = calculate_worked(record, false);
    format!(
        "Session {index}: Stopped at {}. Total today: {}",
        now.format("%H:%M"),
        format_duration(worked),
    )
}
