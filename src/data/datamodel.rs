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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DayRecord {
    pub date: NaiveDate,
    pub events: Vec<Event>,
}
