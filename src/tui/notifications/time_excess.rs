use chrono::Local;

use crate::tui::app_state::App;

pub(crate) fn is_max_time_exceeded(app: &App) -> bool {
    let Some(max_hours) = app.config.max_hours_per_day() else {
        return false;
    };

    let today = Local::now().date_naive();
    let record =
        crate::storage::load_record(today).unwrap_or_else(|| crate::data::DayRecord::new(today));
    let actively_running = record.has_active_session();

    let total_time = crate::tracking_logic::calculate_total_time(&record, actively_running);
    let raw_paused = crate::tracking_logic::calculate_total_paused(&record, actively_running);
    let raw_worked = crate::tracking_logic::calculate_worked(&record, actively_running);
    let effective_paused = crate::tracking_logic::apply_auto_pause(
        raw_worked,
        raw_paused,
        app.config.auto_pause_rules(),
    );
    let effective_worked = total_time - effective_paused;
    let max_duration = chrono::Duration::seconds((max_hours * 3600.0) as i64);

    effective_worked >= max_duration
}
