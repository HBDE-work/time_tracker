use chrono::Duration;
use chrono::Local;
use chrono::NaiveTime;

use crate::data::DayRecord;
use crate::data::EventKind;
use crate::data::Session;

pub(crate) fn calculate_worked(record: &DayRecord, up_to_now: bool) -> Duration {
    let session_count = record.sessions.len();

    record
        .sessions
        .iter()
        .enumerate()
        .map(|(idx, session)| {
            let is_last = idx + 1 == session_count;
            calculate_session_worked(session, up_to_now && is_last)
        })
        .fold(Duration::zero(), |acc, d| acc + d)
}

fn calculate_session_worked(session: &Session, up_to_now: bool) -> Duration {
    let mut total = Duration::zero();
    let mut last_go: Option<NaiveTime> = None;

    for event in &session.events {
        match event.kind {
            EventKind::Go => {
                last_go = Some(event.time);
            }
            EventKind::Pause | EventKind::Stop => {
                if let Some(start) = last_go.take() {
                    total += event.time - start;
                }
            }
        }
    }

    // If there's still an open Go event without a matching Pause/Stop
    if let Some(start) = last_go {
        let end_time = if up_to_now {
            Local::now().time()
        } else {
            NaiveTime::from_hms_opt(23, 59, 59).unwrap()
        };
        total += end_time - start;
    }

    total
}

pub(crate) fn calculate_session_total(session: &Session, up_to_now: bool) -> Duration {
    if session.events.is_empty() {
        return Duration::zero();
    }

    let start_time = session.events.first().unwrap().time;
    let end_time = if up_to_now {
        Local::now().time()
    } else if let Some(last_event) = session.events.last() {
        if last_event.kind == EventKind::Stop {
            last_event.time
        } else {
            NaiveTime::from_hms_opt(23, 59, 59).unwrap()
        }
    } else {
        return Duration::zero();
    };

    end_time - start_time
}

pub(crate) fn calculate_session_paused(session: &Session, up_to_now: bool) -> Duration {
    let mut total = Duration::zero();
    let mut last_pause: Option<NaiveTime> = None;

    for event in &session.events {
        match event.kind {
            EventKind::Pause => {
                last_pause = Some(event.time);
            }
            EventKind::Go => {
                if let Some(pause_start) = last_pause.take() {
                    total += event.time - pause_start;
                }
            }
            EventKind::Stop => {
                if let Some(pause_start) = last_pause.take() {
                    total += event.time - pause_start;
                }
            }
        }
    }

    // If session ends while paused (no Stop after Pause)
    if let Some(pause_start) = last_pause {
        let end_time = if up_to_now {
            Local::now().time()
        } else {
            NaiveTime::from_hms_opt(23, 59, 59).unwrap()
        };
        total += end_time - pause_start;
    }

    total
}

pub(crate) fn calculate_total_time(record: &DayRecord, up_to_now: bool) -> Duration {
    let session_count = record.sessions.len();

    record
        .sessions
        .iter()
        .enumerate()
        .map(|(idx, session)| {
            let is_last = idx + 1 == session_count;
            calculate_session_total(session, up_to_now && is_last)
        })
        .fold(Duration::zero(), |acc, d| acc + d)
}

pub(crate) fn calculate_total_paused(record: &DayRecord, up_to_now: bool) -> Duration {
    let session_count = record.sessions.len();

    record
        .sessions
        .iter()
        .enumerate()
        .map(|(idx, session)| {
            let is_last = idx + 1 == session_count;
            calculate_session_paused(session, up_to_now && is_last)
        })
        .fold(Duration::zero(), |acc, d| acc + d)
}
