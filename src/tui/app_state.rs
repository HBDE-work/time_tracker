use crossterm::event::KeyCode;

use crate::data::EventKind;
use crate::tracking_logic::execute_action;

/// Holds the mutable application state between frames
pub(crate) struct App {
    pub feedback: String,
    pub should_quit: bool,
}

impl App {
    pub(crate) fn new() -> Self {
        Self {
            feedback: String::from("Press a key to execute a command."),
            should_quit: false,
        }
    }

    pub(crate) fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('g') => self.feedback = execute_action(EventKind::Go),
            KeyCode::Char('p') => self.feedback = execute_action(EventKind::Pause),
            KeyCode::Char('s') => self.feedback = execute_action(EventKind::Stop),
            _ => {}
        }
    }
}
