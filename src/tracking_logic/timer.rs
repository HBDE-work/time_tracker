use chrono::Duration;
use chrono::Local;
use chrono::NaiveTime;

use crate::data::DayRecord;
use crate::data::EventKind;

pub(crate) fn calculate_worked(record: &DayRecord, up_to_now: bool) -> Duration {
    let mut total = Duration::zero();
    let mut last_go: Option<NaiveTime> = None;

    for event in &record.events {
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

    // If still running (last event was Go), count up to now
    if up_to_now && let Some(start) = last_go {
        let now = Local::now().time();
        total += now - start;
    }

    total
}
