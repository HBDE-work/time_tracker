use chrono::Local;

use crate::data::glyphs::CLI;
use crate::storage::TrackerConfig;
use crate::storage::load_record;
use crate::tracking_logic::apply_auto_pause;
use crate::tracking_logic::calculate_session_paused;
use crate::tracking_logic::calculate_session_total;
use crate::tracking_logic::calculate_total_paused;
use crate::tracking_logic::calculate_total_time;
use crate::tracking_logic::calculate_worked;
use crate::tracking_logic::format_duration;
use crate::tracking_logic::format_duration_decimal;
use crate::tracking_logic::format_task_summary;
use crate::tracking_logic::resolve_date;

pub(crate) fn cmd_status(day: Option<String>, week: Option<u32>, year: Option<i32>, decimal: bool) {
    let date = resolve_date(day, week, year);

    match load_record(date) {
        None => {
            println!("No record for {}", date.format("%A, %Y-%m-%d"));
        }
        Some(record) => {
            let is_today = date == Local::now().date_naive();
            let still_running = is_today && record.has_active_session();

            let total_time = calculate_total_time(&record, still_running);
            let raw_paused = calculate_total_paused(&record, still_running);
            let raw_worked = calculate_worked(&record, still_running);

            // apply pause rules for the day totals
            let config = TrackerConfig::load();
            let effective_paused =
                apply_auto_pause(raw_worked, raw_paused, config.auto_pause_rules());
            let effective_worked = total_time - effective_paused;
            let auto_pause_bump = effective_paused - raw_paused;

            let format_fn = if decimal {
                format_duration_decimal
            } else {
                format_duration
            };

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

                // session metrics are always raw (auto-pause is a day-level adjustment)
                let session_total = calculate_session_total(session, still_running && is_last);
                let session_paused = calculate_session_paused(session, still_running && is_last);
                let session_worked = session_total - session_paused;

                println!("  ─────────────────────");
                println!("    Total:  {}", format_fn(session_total));
                println!("    Paused: {}", format_fn(session_paused));
                println!("    Worked: {}", format_fn(session_worked));
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
                format_fn(total_time),
                pause_indicator
            );
            let pause_notice = if auto_pause_bump.num_seconds() > 0 {
                // Print pause notice when the rules changed anything
                format!(
                    "  Total Paused: {} ( {} {} pause [was {}, legal minimum {}] )",
                    format_fn(effective_paused),
                    CLI.scales,
                    format_fn(auto_pause_bump),
                    format_fn(raw_paused),
                    format_fn(effective_paused),
                )
            } else {
                format!("  Total Paused: {}", format_fn(effective_paused))
            };
            println!("{pause_notice}");
            println!("  Total Worked: {}", format_fn(effective_worked));

            let task_summary = format_task_summary(&record, effective_worked, decimal);
            if !task_summary.is_empty() {
                print!("{task_summary}");
            }
        }
    }
}
