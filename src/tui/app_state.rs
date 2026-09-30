use crossterm::event::KeyCode;

use crate::data::EventKind;
use crate::data::glyphs::TUI;
use crate::storage::TrackerConfig;
use crate::storage::detect_editor;
use crate::storage::toml_path;
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
    pub editing_max_hours: bool,

    pub active_task: Option<u8>,
    pub decimal_time_format: bool,

    pub history_mode: bool,
    pub viewed_day_offset: i32,

    pub autopause_editor_open: bool,
    pub editing_autopause_buffer: String,
    pub editing_autopause_index: Option<usize>,
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
            feedback: String::from("Press a key to execute a command."),
            should_quit: false,
            reader_status,
            watch_process,
            task_editor_open: false,
            editing_slot: None,
            editing_buffer: String::new(),
            editing_max_hours: false,
            active_task,
            decimal_time_format: config.decimal_time_format(),
            history_mode: false,
            viewed_day_offset: 0,
            autopause_editor_open: false,
            editing_autopause_buffer: String::new(),
            editing_autopause_index: None,
            config,
        }
    }

    pub(crate) fn handle_key(&mut self, code: KeyCode) {
        if self.editing_slot.is_some() || self.editing_max_hours {
            self.handle_key_editing(code);
            return;
        }

        if self.editing_autopause_index.is_some() {
            self.handle_key_autopause_editing(code);
            return;
        }

        if self.task_editor_open {
            self.handle_key_editor_browse(code);
            return;
        }

        if self.autopause_editor_open {
            self.handle_key_autopause_browse(code);
            return;
        }

        self.handle_key_normal(code);
    }

    fn handle_key_normal(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('g') | KeyCode::Char('G') => self.handle_go(),
            KeyCode::Char('p') | KeyCode::Char('P') => self.handle_pause(),
            KeyCode::Char('s') | KeyCode::Char('S') => self.handle_stop(),
            KeyCode::Char('e') | KeyCode::Char('E') => self.handle_edit_day(),
            KeyCode::F(1) => self.toggle_task_editor(),
            KeyCode::F(2) => self.toggle_smartcard(),
            KeyCode::F(3) => self.toggle_history_mode(),
            KeyCode::F(11) => self.toggle_autopause_editor(),
            KeyCode::F(12) => self.toggle_time_format(),
            KeyCode::Left if self.history_mode => {
                self.navigate_history(-1);
            }

            KeyCode::Right if self.history_mode => {
                self.navigate_history(1);
            }

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
            KeyCode::Char('h') | KeyCode::Char('H') => {
                self.editing_buffer = self
                    .config
                    .max_hours_per_day()
                    .map(|h| format!("{:.2}", h))
                    .unwrap_or_default();
                self.editing_max_hours = true;
                self.feedback =
                    "Editing max hours/day - type hours (e.g. 8.5), Enter to save, Esc to cancel"
                        .into();
            }
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                let slot = ch as u8 - b'0';
                // Build editing buffer with task name and optional max hours
                if let Some(task_info) = self.config.task_info(slot) {
                    self.editing_buffer = task_info.name.clone();
                    if let Some(max_hours) = task_info.max_hours {
                        self.editing_buffer.push_str(&format!(":{:.2}", max_hours));
                    }
                } else {
                    self.editing_buffer.clear();
                }
                self.editing_slot = Some(slot);
                self.feedback = format!(
                    "Editing slot {slot} - format: TaskName or TaskName:2.5 - Enter to save, Esc to cancel"
                );
            }
            _ => {}
        }
    }

    fn handle_key_editing(&mut self, code: KeyCode) {
        match code {
            KeyCode::Enter => {
                if self.editing_max_hours {
                    // Save max hours
                    let trimmed = self.editing_buffer.trim();
                    if trimmed.is_empty() {
                        self.config.set_max_hours_per_day(None);
                        self.feedback = "Max hours/day cleared.".into();
                    } else {
                        match trimmed.parse::<f64>() {
                            Ok(hours) if hours > 0.0 && hours <= 24.0 => {
                                self.config.set_max_hours_per_day(Some(hours));
                                self.feedback = format!("Max hours/day set to {:.2}h", hours);
                            }
                            _ => {
                                self.feedback =
                                    "Invalid hours. Enter a number between 0 and 24.".into();
                            }
                        }
                    }
                    self.editing_max_hours = false;
                    self.editing_buffer.clear();
                    self.save_config();
                } else if let Some(slot) = self.editing_slot.take() {
                    // Save task slot
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
            }
            KeyCode::Esc => {
                if self.editing_max_hours {
                    self.editing_max_hours = false;
                    self.feedback = "Editing max hours cancelled.".into();
                } else if let Some(slot) = self.editing_slot.take() {
                    self.feedback = format!("Editing slot {slot} cancelled.");
                }
                self.editing_buffer.clear();
            }
            KeyCode::Backspace => {
                self.editing_buffer.pop();
            }
            KeyCode::Char(ch) => {
                // For max hours, only allow digits, decimal point
                if self.editing_max_hours {
                    if ch.is_ascii_digit() || ch == '.' {
                        self.editing_buffer.push(ch);
                    }
                } else {
                    self.editing_buffer.push(ch);
                }
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

    pub(crate) fn handle_edit_day(&mut self) {
        // Check if editor is available
        if detect_editor().is_none() {
            self.feedback = "No text editor found. Set EDITOR environment variable.".into();
            return;
        }

        let viewed_date = self.get_viewed_date();
        let file_path = toml_path(viewed_date);

        // Create file if it doesn't exist
        if !file_path.exists() {
            use crate::data::DayRecord;
            use crate::storage::save_record;
            let empty_record = DayRecord::new(viewed_date);
            save_record(&empty_record);
        }

        self.feedback = format!("Opening {} in editor...", viewed_date.format("%Y-%m-%d"));
    }

    pub(crate) fn get_toml_path(&self) -> std::path::PathBuf {
        toml_path(self.get_viewed_date())
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

    fn toggle_autopause_editor(&mut self) {
        if self.autopause_editor_open {
            self.editing_autopause_index = None;
            self.editing_autopause_buffer.clear();
        }
        self.autopause_editor_open = !self.autopause_editor_open;

        if self.autopause_editor_open {
            self.feedback =
                "Pause Editor: [N] new rule  [1-9] edit rule  [D+digit] delete  [Esc/F11] close"
                    .into();
        } else {
            self.feedback = "Auto-Pause Editor closed.".into();
        }
    }

    fn handle_key_autopause_browse(&mut self, code: KeyCode) {
        match code {
            KeyCode::F(11) | KeyCode::Esc => self.toggle_autopause_editor(),
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.editing_autopause_index = Some(usize::MAX); // new rule mode
                self.editing_autopause_buffer.clear();
                self.feedback =
                    "New rule: type HOURS:MINUTES (e.g. 8:30), Enter to save, Esc to cancel".into();
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                self.feedback = "Delete rule: press the rule number to remove it.".into();
                self.editing_autopause_index = Some(usize::MAX - 1); // delete mode
            }
            KeyCode::Char(ch) if ch.is_ascii_digit() && ch != '0' => {
                let idx = (ch as u8 - b'1') as usize;

                // if delete mode
                if self.editing_autopause_index == Some(usize::MAX - 1) {
                    let mut rules = self.config.auto_pause_rules().to_owned();
                    if idx < rules.len() {
                        let removed = rules.remove(idx);
                        self.config.set_auto_pause_rules(rules);
                        self.save_config();
                        self.feedback = format!(
                            "Rule {} deleted (after {}h -> {}min pause).",
                            idx + 1,
                            removed.after_hours,
                            removed.minimum_pause_minutes
                        );
                    } else {
                        self.feedback = format!("No rule {} to delete.", idx + 1);
                    }
                    self.editing_autopause_index = None;
                    return;
                }

                let rules = self.config.auto_pause_rules();
                if idx < rules.len() {
                    let rule = &rules[idx];
                    self.editing_autopause_buffer =
                        format!("{}:{}", rule.after_hours, rule.minimum_pause_minutes);
                    self.editing_autopause_index = Some(idx);
                    self.feedback = format!(
                        "Editing rule {}: HOURS:MINUTES, Enter to save, Esc to cancel",
                        idx + 1
                    );
                } else {
                    self.feedback = format!("No rule at slot {}.", idx + 1);
                }
            }
            _ => {}
        }
    }

    fn handle_key_autopause_editing(&mut self, code: KeyCode) {
        match code {
            KeyCode::Enter => {
                let raw = self.editing_autopause_buffer.trim().to_owned();
                match parse_autopause_rule(&raw) {
                    Ok((after_hours, minimum_pause_minutes)) => {
                        use crate::storage::AutoPauseRule;
                        let new_rule = AutoPauseRule::new(after_hours, minimum_pause_minutes);
                        let mut rules = self.config.auto_pause_rules().to_owned();

                        let slot = self.editing_autopause_index.unwrap_or(usize::MAX);
                        if slot == usize::MAX {
                            rules.push(new_rule);
                            // keep rules sorted by threshold
                            rules.sort_by(|rule_a, rule_b| {
                                rule_a
                                    .after_hours
                                    .partial_cmp(&rule_b.after_hours)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            });
                            self.feedback = format!(
                                "Auto-pause rule added: after {after_hours}h -> {minimum_pause_minutes}min pause."
                            );
                        } else {
                            if slot < rules.len() {
                                rules[slot] = new_rule;
                                rules.sort_by(|rule_a, rule_b| {
                                    rule_a
                                        .after_hours
                                        .partial_cmp(&rule_b.after_hours)
                                        .unwrap_or(std::cmp::Ordering::Equal)
                                });
                            }
                            self.feedback = format!(
                                "Rule updated: after {after_hours}h -> {minimum_pause_minutes}min pause."
                            );
                        }

                        self.config.set_auto_pause_rules(rules);
                        self.save_config();
                    }
                    Err(hint) => {
                        self.feedback = format!("Invalid format: {hint}");
                    }
                }
                self.editing_autopause_index = None;
                self.editing_autopause_buffer.clear();
            }
            KeyCode::Esc => {
                self.editing_autopause_index = None;
                self.editing_autopause_buffer.clear();
                self.feedback = "Edit cancelled.".into();
            }
            KeyCode::Backspace => {
                self.editing_autopause_buffer.pop();
            }
            KeyCode::Char(ch) if ch.is_ascii_digit() || ch == '.' || ch == ':' => {
                self.editing_autopause_buffer.push(ch);
            }
            _ => {}
        }
    }

    fn toggle_task_editor(&mut self) {
        if self.task_editor_open {
            self.editing_slot = None;
            self.editing_max_hours = false;
            self.editing_buffer.clear();
        }
        self.task_editor_open = !self.task_editor_open;
        self.save_config();

        if self.task_editor_open {
            self.feedback =
                "Task Editor opened. Press 0 - 9 to edit a slot, H for max hours.".into();
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

    fn toggle_time_format(&mut self) {
        self.decimal_time_format = !self.decimal_time_format;
        self.config
            .set_decimal_time_format(self.decimal_time_format);
        if self.decimal_time_format {
            self.feedback = "Time format: Decimal hours".into();
        } else {
            self.feedback = "Time format: Hours and minutes".into();
        }
        self.save_config();
    }

    fn toggle_history_mode(&mut self) {
        self.history_mode = !self.history_mode;
        if self.history_mode {
            self.feedback = "History mode: ON - Use <- | -> to navigate days".into();
        } else {
            self.viewed_day_offset = 0;
            self.feedback = "History mode: OFF - Viewing today".into();
        }
    }

    fn navigate_history(&mut self, direction: i32) {
        self.viewed_day_offset += direction;

        // Don't allow viewing future dates
        if self.viewed_day_offset > 0 {
            self.viewed_day_offset = 0;
        }

        let viewed_date = self.get_viewed_date();
        self.feedback = format!("Viewing: {}", viewed_date.format("%A, %Y-%m-%d"));
    }

    pub(crate) fn get_viewed_date(&self) -> chrono::NaiveDate {
        use chrono::Local;
        let today = Local::now().date_naive();
        today + chrono::Duration::days(self.viewed_day_offset as i64)
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

/// Parse a rule string of the form "HOURS:MINUTES" (e.g. "8:30" or "8.5:45")
///
/// Returns `(after_hours, minimum_pause_minutes)` or an error hint string
fn parse_autopause_rule(input: &str) -> Result<(f64, u64), String> {
    let (hours_part, minutes_part) = input
        .split_once(':')
        .ok_or_else(|| "expected format HOURS:MINUTES (e.g. 8:30)".to_string())?;

    let after_hours = hours_part
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("'{hours_part}' is not a valid number of hours"))?;

    let minimum_pause_minutes = minutes_part
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("'{minutes_part}' is not a valid number of minutes"))?;

    if after_hours <= 0.0 || after_hours > 24.0 {
        return Err(format!("hours must be between 0 and 24, got {after_hours}"));
    }
    if minimum_pause_minutes == 0 || minimum_pause_minutes > 480 {
        return Err(format!(
            "pause minutes must be between 1 and 480, got {minimum_pause_minutes}"
        ));
    }

    Ok((after_hours, minimum_pause_minutes))
}
