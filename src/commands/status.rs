use chrono::Local;

use crate::data::glyphs::CLI;
use crate::storage::load_record;
use crate::tracking_logic::calculate_worked;
use crate::tracking_logic::format_duration;
use crate::tracking_logic::format_task_summary;
use crate::tracking_logic::resolve_date;

pub(crate) fn cmd_status(day: Option<String>, week: Option<u32>, year: Option<i32>) {
    let date = resolve_date(day, week, year);

    match load_record(date) {
        None => {
            println!("No record for {}", date.format("%A, %Y-%m-%d"));
        }
        Some(record) => {
            let is_today = date == Local::now().date_naive();
            let still_running = is_today && record.has_active_session();
            let worked = calculate_worked(&record, still_running);

            let rule = CLI.horizontal_rule;
            println!("{rule} {} {rule}", date.format("%A, %Y-%m-%d"));

            for session in &record.sessions {
                let stopped = session.is_stopped();
                let active = is_today && session.is_active();

                let state = if active {
                    " (tracking)"
                } else if stopped {
                    " (stopped)"
                } else {
                    " (paused)"
                };

                println!("\n  Session {}{state}", session.index);
                for event in &session.events {
                    println!("    {} {}", event.time.format("%H:%M"), event.kind);
                }
            }

            if still_running {
                println!("\n  Currently tracking");
            }

            println!("\n  Total: {}", format_duration(worked));

            let task_summary = format_task_summary(&record, worked);
            if !task_summary.is_empty() {
                print!("{task_summary}");
            }
        }
    }
}
