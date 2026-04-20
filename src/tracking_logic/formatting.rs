/// Format a duration as hours and minutes
pub(crate) fn format_duration(duration: chrono::Duration) -> String {
    let total_minutes = duration.num_minutes();
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{hours}h {minutes:02}m")
}

/// Format a duration as decimal hours
pub(crate) fn format_duration_decimal(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();
    let hours = total_seconds as f64 / 3600.0;
    format!("{:.2}h", hours)
}
