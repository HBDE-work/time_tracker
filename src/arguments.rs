use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "Time Tracker")]
#[command(version, about = "Simple CLI Timetracker", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start or resume tracking
    Go,
    /// Pause the current tracking
    Pause,
    /// Stop tracking for today
    Stop,
    /// Show tracked hours (today by default, or a specific weekday of a given week/year)
    Status {
        /// Weekday name (e.g. "monday", "tue")
        day: Option<String>,
        /// Calendar week number (1-53, defaults to current week)
        week: Option<u32>,
        /// Year (defaults to current year)
        year: Option<i32>,
    },
    /// Launch interactive terminal UI
    Tui,
}
