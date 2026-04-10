use chrono::Local;

use crate::data::EventKind;
use crate::storage::load_record;
use crate::tracking_logic::calculate_worked;
use crate::tracking_logic::format_duration;
use crate::tracking_logic::format_task_summary;
use crate::tracking_logic::last_event_kind;
use crate::tracking_logic::resolve_date;

pub(crate) fn cmd_status(day: Option<String>, week: Option<u32>, year: Option<i32>) {
    let date = resolve_date(day, week, year);

    match load_record(date) {
        None => {
            println!("No record for {}", date.format("%A, %Y-%m-%d"));
        }
        Some(record) => {
            let is_today = date == Local::now().date_naive();
            let still_running = last_event_kind(&record) == Some(&EventKind::Go);
            let worked = calculate_worked(&record, is_today && still_running);

            println!("── {} ──", date.format("%A, %Y-%m-%d"));
            for event in &record.events {
                println!("  {} {}", event.time.format("%H:%M"), event.kind);
            }
            if is_today && still_running {
                println!("  ⏳  currently tracking");
            }
            println!("\n  Total: {}", format_duration(worked));

            // Task breakdown (only printed when task events exist)
            let task_summary = format_task_summary(&record, worked);
            if !task_summary.is_empty() {
                print!("{task_summary}");
            }
        }
    }
}
