use crossterm::event::KeyCode;

use crate::data::EventKind;
use crate::storage::TrackerConfig;
use crate::tracking_logic::active_task_name;
use crate::tracking_logic::execute_action;
use crate::tracking_logic::start_task;
use crate::tracking_logic::stop_active_task;
use crate::tracking_logic::today_record;

use super::smartcard::CardEvent;
use super::smartcard::ReaderProbe;
use super::smartcard::SmartcardWatchProcess;

/// Holds the mutable application state between frames
pub(crate) struct App {
    pub config: TrackerConfig,

    pub feedback: String,
    pub should_quit: bool,

    /// Result of the initial reader probe (cached at startup)
    pub reader_status: ReaderProbe,
    /// Background Process handle: only `Some` while the feature is active
    watch_process: Option<SmartcardWatchProcess>,

    // Task editor state
    /// `true` while the lower panel shows the task-name editor
    pub task_editor_open: bool,
    /// Which slot (0 - 9) the user is currently typing a name for
    pub editing_slot: Option<u8>,
    /// In-progress text for the slot being edited
    pub editing_buffer: String,

    // Active task tracking
    /// Slot number of the task currently being time-tracked (`None` = global)
    pub active_task: Option<u8>,
}

impl App {
    pub(crate) fn new() -> Self {
        let reader_status = super::smartcard::probe_readers();
        let mut config = TrackerConfig::load();

        // Restore the smartcard watcher from config
        // if a reader is still available
        let watch_process = if config.smartcard_active() && reader_status == ReaderProbe::Available
        {
            Some(SmartcardWatchProcess::spawn())
        } else {
            if config.smartcard_active() {
                // Reader disappeared since last run; reset so we don't keep
                // writing a stale "active" flag.
                config.set_smartcard_active(false);
                let _ = config.save();
            }
            None
        };

        // Check if a task was left open from a previous session
        let active_task = Self::recover_active_task(&config);

        Self {
            config,
            feedback: String::from("Press a key to execute a command."),
            should_quit: false,
            reader_status,
            watch_process,
            task_editor_open: false,
            editing_slot: None,
            editing_buffer: String::new(),
            active_task,
        }
    }

    pub(crate) fn handle_key(&mut self, code: KeyCode) {
        // When actively editing a slot name, consume most keys for text input
        if self.editing_slot.is_some() {
            self.handle_key_editing(code);
            return;
        }

        // When the editor panel is open (but not typing in a slot)
        if self.task_editor_open {
            self.handle_key_editor_browse(code);
            return;
        }

        // Normal mode
        self.handle_key_normal(code);
    }

    /// Normal TUI mode - GPS commands, F-keys, and task number toggles
    fn handle_key_normal(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('g') => self.handle_go(),
            KeyCode::Char('p') => self.handle_pause(),
            KeyCode::Char('s') => self.handle_stop(),
            KeyCode::F(1) => self.toggle_task_editor(),
            KeyCode::F(2) => self.toggle_smartcard(),
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                let slot = ch as u8 - b'0';
                self.toggle_task(slot);
            }
            _ => {}
        }
    }

    /// Editor is visible but user hasn't selected a slot yet
    fn handle_key_editor_browse(&mut self, code: KeyCode) {
        match code {
            KeyCode::F(1) | KeyCode::Esc => self.toggle_task_editor(),
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                let slot = ch as u8 - b'0';
                // Pre-fill buffer with existing name (if any)
                self.editing_buffer = self.config.task_name(slot).unwrap_or("").to_owned();
                self.editing_slot = Some(slot);
                self.feedback =
                    format!("Editing slot {slot} - type a name, Enter to save, Esc to cancel");
            }
            _ => {}
        }
    }

    /// User is typing a task name into a specific slot
    fn handle_key_editing(&mut self, code: KeyCode) {
        match code {
            KeyCode::Enter => {
                let slot = self.editing_slot.take().unwrap();
                let name = self.editing_buffer.trim().to_owned();
                self.config.set_task_name(slot, &name);
                self.save_config();
                if name.is_empty() {
                    self.feedback = format!("Slot {slot} cleared.");
                    // If the cleared slot was the active task, return to global
                    if self.active_task == Some(slot) {
                        self.active_task = None;
                    }
                } else {
                    self.feedback = format!("Slot {slot} -> \"{name}\" saved.");
                }
                self.editing_buffer.clear();
            }
            KeyCode::Esc => {
                let slot = self.editing_slot.take().unwrap();
                self.editing_buffer.clear();
                self.feedback = format!("Editing slot {slot} cancelled.");
            }
            KeyCode::Backspace => {
                self.editing_buffer.pop();
            }
            KeyCode::Char(ch) => {
                self.editing_buffer.push(ch);
            }
            _ => {}
        }
    }

    /// `[g]` Go: if a task is active, this is a no-op (already tracking)
    ///
    /// If paused/stopped while a task was selected, resume that task
    fn handle_go(&mut self) {
        let record = today_record();
        let already_going = crate::tracking_logic::last_event_kind(&record) == Some(&EventKind::Go);

        if already_going {
            self.feedback = "Already tracking.".into();
            return;
        }

        // Fire the global Go event first
        let base_msg = execute_action(EventKind::Go);

        // If a task slot is active, also (re)open its timespan
        if let Some(slot) = self.active_task
            && let Some(name) = self.config.task_name(slot)
        {
            let task_msg = start_task(name);
            self.feedback = format!("{base_msg} | {task_msg}");
            return;
        }
        self.feedback = base_msg;
    }

    /// `[p]` Pause: pauses tracking
    ///
    /// If a task is active, its timespan is closed
    ///
    /// but the slot stays selected so `[g]` resumes it
    fn handle_pause(&mut self) {
        // Close any open task timespan (but keep slot selected)
        let task_part = stop_active_task();
        let base_msg = execute_action(EventKind::Pause);

        self.feedback = match task_part {
            Some(tmsg) => format!("{tmsg} | {base_msg}"),
            None => base_msg,
        };
    }

    /// `[s]` Stop: stops tracking for the day and Clears active task
    fn handle_stop(&mut self) {
        let task_part = stop_active_task();
        self.active_task = None;
        let base_msg = execute_action(EventKind::Stop);

        self.feedback = match task_part {
            Some(tmsg) => format!("{tmsg} | {base_msg}"),
            None => base_msg,
        };
    }

    /// Press a digit key to start/switch/stop a task
    fn toggle_task(&mut self, slot: u8) {
        let Some(name) = self.config.task_name(slot) else {
            self.feedback = format!("Slot {slot} has no task assigned. Press F1 to edit.");
            return;
        };

        // Same slot pressed again -> back to global time
        if self.active_task == Some(slot) {
            if let Some(msg) = stop_active_task() {
                self.feedback = msg;
            }
            self.active_task = None;
            return;
        }

        // Different slot (or no active task) -> ensure global tracking is on,
        // then start the new task
        let record = today_record();
        let not_going = crate::tracking_logic::last_event_kind(&record) != Some(&EventKind::Go);
        if not_going {
            // Auto-start global tracking when user picks a task
            let _ = execute_action(EventKind::Go);
        }

        self.feedback = start_task(name);
        self.active_task = Some(slot);
    }

    /// Toggle the task editor panel on/off
    fn toggle_task_editor(&mut self) {
        // If closing, discard any in-progress edit
        if self.task_editor_open {
            self.editing_slot = None;
            self.editing_buffer.clear();
        }
        self.task_editor_open = !self.task_editor_open;
        self.save_config();

        if self.task_editor_open {
            self.feedback = "Task Editor opened. Press 0 - 9 to edit a slot.".into();
        } else {
            self.feedback = "Task Editor closed.".into();
        }
    }

    /// Toggle the smartcard auto-tracking feature on/off
    fn toggle_smartcard(&mut self) {
        if self.config.smartcard_active() {
            // Turn off which stops the watch process thread
            if let Some(watcher) = self.watch_process.take() {
                watcher.stop();
            }
            self.config.set_smartcard_active(false);
            self.save_config();
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
                self.config.set_smartcard_active(true);
                self.save_config();
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

    /// Persist config and show any error on the feedback line
    fn save_config(&mut self) {
        if let Err(err) = self.config.save() {
            self.feedback = format!("Error: {err}");
        }
    }

    /// On startup, check if a task was left running (open timespan) and map it
    /// back to its config slot number
    fn recover_active_task(config: &TrackerConfig) -> Option<u8> {
        let record = today_record();
        let running_name = active_task_name(&record)?;

        // Walk configured slots to find one matching the open timespan's name
        (0u8..=9).find(|&slot| config.task_name(slot) == Some(running_name))
    }

    /// Drain any pending card events from the background watch process and fire the
    /// corresponding tracker actions.
    ///
    /// Routes through the task-aware handle_go / handle_pause so that an
    /// active task is properly resumed / paused on card insert / remove.
    ///
    /// Call this once per frame.
    pub(crate) fn process_card_events(&mut self) {
        let Some(watcher) = self.watch_process.as_ref() else {
            return;
        };

        // Drain all queued events into a local buffer to release the
        // immutable borrow on `self` before we call &mut self methods.
        let events: Vec<CardEvent> = watcher.rx.try_iter().collect();

        for evt in events {
            match evt {
                CardEvent::Inserted => {
                    self.handle_go();
                    self.feedback = format!("🔑 Card inserted - {}", self.feedback);
                }
                CardEvent::Removed => {
                    self.handle_pause();
                    self.feedback = format!("🔑 Card removed - {}", self.feedback);
                }
            }
        }
    }
}
