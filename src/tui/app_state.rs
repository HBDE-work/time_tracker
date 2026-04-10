use crossterm::event::KeyCode;

use crate::data::EventKind;
use crate::data::glyphs::TUI;
use crate::storage::TrackerConfig;
use crate::tracking_logic::active_task_name;
use crate::tracking_logic::execute_action;
use crate::tracking_logic::start_task;
use crate::tracking_logic::stop_active_task;
use crate::tracking_logic::today_record;

use super::smartcard::CardEvent;
use super::smartcard::ReaderProbe;
use super::smartcard::SmartcardWatchProcess;

pub(crate) struct App {
    pub config: TrackerConfig,

    pub feedback: String,
    pub should_quit: bool,

    pub reader_status: ReaderProbe,
    watch_process: Option<SmartcardWatchProcess>,

    pub task_editor_open: bool,
    pub editing_slot: Option<u8>,
    pub editing_buffer: String,

    pub active_task: Option<u8>,
}

impl App {
    pub(crate) fn new() -> Self {
        let reader_status = super::smartcard::probe_readers();
        let mut config = TrackerConfig::load();

        let watch_process = if config.smartcard_active() && reader_status == ReaderProbe::Available
        {
            Some(SmartcardWatchProcess::spawn())
        } else {
            if config.smartcard_active() {
                config.set_smartcard_active(false);
                let _ = config.save();
            }
            None
        };

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
        if self.editing_slot.is_some() {
            self.handle_key_editing(code);
            return;
        }

        if self.task_editor_open {
            self.handle_key_editor_browse(code);
            return;
        }

        self.handle_key_normal(code);
    }

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

    fn handle_key_editor_browse(&mut self, code: KeyCode) {
        match code {
            KeyCode::F(1) | KeyCode::Esc => self.toggle_task_editor(),
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                let slot = ch as u8 - b'0';
                self.editing_buffer = self.config.task_name(slot).unwrap_or("").to_owned();
                self.editing_slot = Some(slot);
                self.feedback =
                    format!("Editing slot {slot} - type a name, Enter to save, Esc to cancel");
            }
            _ => {}
        }
    }

    fn handle_key_editing(&mut self, code: KeyCode) {
        match code {
            KeyCode::Enter => {
                let slot = self.editing_slot.take().unwrap();
                let name = self.editing_buffer.trim().to_owned();
                self.config.set_task_name(slot, &name);
                self.save_config();
                if name.is_empty() {
                    self.feedback = format!("Slot {slot} cleared.");
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

    fn handle_go(&mut self) {
        let record = today_record();
        if record.current_session().is_some_and(|s| s.is_active()) {
            self.feedback = "Already tracking.".into();
            return;
        }

        let base_msg = execute_action(EventKind::Go);

        if let Some(slot) = self.active_task
            && let Some(name) = self.config.task_name(slot)
        {
            let task_msg = start_task(name);
            self.feedback = format!("{base_msg} | {task_msg}");
            return;
        }
        self.feedback = base_msg;
    }

    fn handle_pause(&mut self) {
        let task_part = stop_active_task();
        let base_msg = execute_action(EventKind::Pause);

        self.feedback = match task_part {
            Some(tmsg) => format!("{tmsg} | {base_msg}"),
            None => base_msg,
        };
    }

    fn handle_stop(&mut self) {
        let task_part = stop_active_task();
        self.active_task = None;
        let base_msg = execute_action(EventKind::Stop);

        self.feedback = match task_part {
            Some(tmsg) => format!("{tmsg} | {base_msg}"),
            None => base_msg,
        };
    }

    fn toggle_task(&mut self, slot: u8) {
        let Some(name) = self.config.task_name(slot) else {
            self.feedback = format!("Slot {slot} has no task assigned. Press F1 to edit.");
            return;
        };

        if self.active_task == Some(slot) {
            if let Some(msg) = stop_active_task() {
                self.feedback = msg;
            }
            self.active_task = None;
            return;
        }

        let record = today_record();
        let needs_go = !record.has_active_session();
        if needs_go {
            let _ = execute_action(EventKind::Go);
        }

        self.feedback = start_task(name);
        self.active_task = Some(slot);
    }

    fn toggle_task_editor(&mut self) {
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

    fn toggle_smartcard(&mut self) {
        if self.config.smartcard_active() {
            if let Some(watcher) = self.watch_process.take() {
                watcher.stop();
            }
            self.config.set_smartcard_active(false);
            self.save_config();
            self.feedback = "Smartcard auto-tracking OFF.".into();
            return;
        }

        if self.reader_status == ReaderProbe::Unavailable {
            self.feedback = "Cannot activate: PC/SC library not found on this system.".into();
            return;
        }

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

    fn save_config(&mut self) {
        if let Err(err) = self.config.save() {
            self.feedback = format!("Error: {err}");
        }
    }

    fn recover_active_task(config: &TrackerConfig) -> Option<u8> {
        let record = today_record();
        let running_name = active_task_name(&record)?;
        (0u8..=9).find(|&slot| config.task_name(slot) == Some(running_name))
    }

    pub(crate) fn process_card_events(&mut self) {
        let Some(watcher) = self.watch_process.as_ref() else {
            return;
        };

        let events: Vec<CardEvent> = watcher.rx.try_iter().collect();

        for evt in events {
            match evt {
                CardEvent::Inserted => {
                    self.handle_go();
                    self.feedback = format!("{} Card inserted - {}", TUI.smartcard, self.feedback);
                }
                CardEvent::Removed => {
                    self.handle_pause();
                    self.feedback = format!("{} Card removed - {}", TUI.smartcard, self.feedback);
                }
            }
        }
    }
}
