use crossterm::event::KeyCode;

use crate::data::EventKind;
use crate::tracking_logic::execute_action;

use super::smartcard::CardEvent;
use super::smartcard::ReaderProbe;
use super::smartcard::SmartcardWatchProcess;

/// Holds the mutable application state between frames
pub(crate) struct App {
    pub feedback: String,
    pub should_quit: bool,

    /// Whether the smartcard auto-tracking feature is active
    pub smartcard_active: bool,
    /// Result of the initial reader probe (cached at startup)
    pub reader_status: ReaderProbe,
    /// Background Process handle: only `Some` while the feature is active
    watch_process: Option<SmartcardWatchProcess>,
}

impl App {
    pub(crate) fn new() -> Self {
        let reader_status = super::smartcard::probe_readers();
        Self {
            feedback: String::from("Press a key to execute a command."),
            should_quit: false,
            smartcard_active: false,
            reader_status,
            watch_process: None,
        }
    }

    pub(crate) fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('g') => self.feedback = execute_action(EventKind::Go),
            KeyCode::Char('p') => self.feedback = execute_action(EventKind::Pause),
            KeyCode::Char('s') => self.feedback = execute_action(EventKind::Stop),
            KeyCode::F(2) => self.toggle_smartcard(),
            _ => {}
        }
    }

    /// Toggle the smartcard auto-tracking feature on/off
    fn toggle_smartcard(&mut self) {
        if self.smartcard_active {
            // Turn off which stops the watch process thread
            if let Some(watcher) = self.watch_process.take() {
                watcher.stop();
            }
            self.smartcard_active = false;
            self.feedback = "Smartcard auto-tracking OFF.".into();
            return;
        }

        // If the library itself is missing there is nothing to re-probe
        if self.reader_status == ReaderProbe::Unavailable {
            self.feedback = "Cannot activate: PC/SC library not found on this system.".into();
            return;
        }

        // Re-probe so we catch readers that were plugged in after startup
        self.reader_status = super::smartcard::probe_readers();

        match self.reader_status {
            ReaderProbe::Available => {
                self.watch_process = Some(SmartcardWatchProcess::spawn());
                self.smartcard_active = true;
                self.feedback = "Smartcard auto-tracking ON.".into();
            }
            ReaderProbe::NoReaders => {
                self.feedback = "Cannot activate: no smartcard reader detected.".into();
            }
            ReaderProbe::Unavailable => {
                self.feedback = "Cannot activate: PC/SC subsystem unavailable.".into();
            }
        }
    }

    /// Drain any pending card events from the background watch process and fire the
    /// corresponding tracker actions
    ///
    /// Call this once per frame
    pub(crate) fn process_card_events(&mut self) {
        let Some(watcher) = self.watch_process.as_ref() else {
            return;
        };

        // Drain all queued events
        // only the last one matters for feedback but
        // we fire an action for each transition so the record is accurate
        while let Ok(evt) = watcher.rx.try_recv() {
            match evt {
                CardEvent::Inserted => {
                    self.feedback =
                        format!("🔑 Card inserted - {}", execute_action(EventKind::Go),);
                }
                CardEvent::Removed => {
                    self.feedback =
                        format!("🔑 Card removed - {}", execute_action(EventKind::Pause),);
                }
            }
        }
    }
}
