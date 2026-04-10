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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TaskEvent {
    pub task: String,
    pub start: NaiveTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<NaiveTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Session {
    pub index: u32,
    pub events: Vec<Event>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub task_events: Vec<TaskEvent>,
}

impl Session {
    pub(crate) fn new(index: u32) -> Self {
        Self {
            index,
            events: Vec::new(),
            task_events: Vec::new(),
        }
    }

    pub(crate) fn last_event_kind(&self) -> Option<&EventKind> {
        self.events.last().map(|e| &e.kind)
    }

    pub(crate) fn is_stopped(&self) -> bool {
        self.last_event_kind() == Some(&EventKind::Stop)
    }

    pub(crate) fn is_active(&self) -> bool {
        self.last_event_kind() == Some(&EventKind::Go)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DayRecord {
    pub date: NaiveDate,
    pub sessions: Vec<Session>,
}

impl DayRecord {
    pub(crate) fn new(date: NaiveDate) -> Self {
        Self {
            date,
            sessions: Vec::new(),
        }
    }

    pub(crate) fn current_session(&self) -> Option<&Session> {
        self.sessions.last()
    }

    pub(crate) fn current_session_mut(&mut self) -> Option<&mut Session> {
        self.sessions.last_mut()
    }

    pub(crate) fn next_session_index(&self) -> u32 {
        self.sessions.last().map(|s| s.index + 1).unwrap_or(0)
    }

    pub(crate) fn has_active_session(&self) -> bool {
        self.current_session().is_some_and(|s| !s.is_stopped())
    }
}
