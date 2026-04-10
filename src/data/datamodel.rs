use chrono::NaiveDate;
use chrono::NaiveTime;
use ratatui::style::Color;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum EventKind {
    Go,
    Pause,
    Stop,
}

impl EventKind {
    pub(crate) fn color(&self) -> Color {
        match self {
            EventKind::Go => Color::Green,
            EventKind::Pause => Color::Yellow,
            EventKind::Stop => Color::Red,
        }
    }
}

impl std::fmt::Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventKind::Go => write!(f, "Go"),
            EventKind::Pause => write!(f, "Pause"),
            EventKind::Stop => write!(f, "Stop"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Event {
    pub kind: EventKind,
    pub time: NaiveTime,
}

/// A single timespan spent on a named task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TaskEvent {
    /// Human-readable task name (not the slot number)
    pub task: String,
    pub start: NaiveTime,
    /// `None` while the task is still running
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<NaiveTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DayRecord {
    pub date: NaiveDate,
    pub events: Vec<Event>,

    /// Named-task timespans (absent / empty in legacy files)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub task_events: Vec<TaskEvent>,
}
