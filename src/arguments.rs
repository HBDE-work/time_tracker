use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "Time Tracker")]
#[command(version, about = "Simple CLI Timetracker", long_about = None)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Start or resume tracking (optionally for a named task)
    Go {
        /// Track a named task (e.g. --task "Code Review")
        #[arg(short, long)]
        task: Option<String>,
    },
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
        /// Display time in decimal hours format
        #[arg(short, long)]
        decimal: bool,
    },
    /// Launch interactive terminal UI
    Tui,
}
