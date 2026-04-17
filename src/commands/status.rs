use chrono::Local;

use crate::data::glyphs::CLI;
use crate::storage::load_record;
use crate::tracking_logic::calculate_session_paused;
use crate::tracking_logic::calculate_session_total;
use crate::tracking_logic::calculate_total_paused;
use crate::tracking_logic::calculate_total_time;
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

            let total_time = calculate_total_time(&record, still_running);
            let total_paused = calculate_total_paused(&record, still_running);
            let total_worked = calculate_worked(&record, still_running);

            let rule = CLI.horizontal_rule;
            println!("{rule} {} {rule}", date.format("%A, %Y-%m-%d"));

            let session_count = record.sessions.len();
            for (idx, session) in record.sessions.iter().enumerate() {
                let is_last = idx + 1 == session_count;
                let stopped = session.is_stopped();
                let active = is_today && is_last && session.is_active();

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

                // Calculate and display session metrics
                let session_total = calculate_session_total(session, still_running && is_last);
                let session_paused = calculate_session_paused(session, still_running && is_last);
                let session_worked = session_total - session_paused;

                println!("  ─────────────────────");
                println!("    Total:  {}", format_duration(session_total));
                println!("    Paused: {}", format_duration(session_paused));
                println!("    Worked: {}", format_duration(session_worked));
            }

            let pause_indicator = if is_today && record.is_paused() {
                " (paused)"
            } else if is_today && record.is_tracking() {
                " (tracking)"
            } else {
                ""
            };

            println!("\n  ═════════════════════");
            println!(
                "  Total Time:   {}{}",
                format_duration(total_time),
                pause_indicator
            );
            println!("  Total Paused: {}", format_duration(total_paused));
            println!("  Total Worked: {}", format_duration(total_worked));

            let task_summary = format_task_summary(&record, total_worked);
            if !task_summary.is_empty() {
                print!("{task_summary}");
            }
        }
    }
}
