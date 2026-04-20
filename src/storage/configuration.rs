use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = ".tracker_options.toml";

pub(super) fn data_dir() -> PathBuf {
    let base = if cfg!(windows) {
        std::env::var("APPDATA").expect("APPDATA not set")
    } else {
        let home = std::env::var("HOME").expect("HOME not set");
        format!("{home}/.config")
    };
    PathBuf::from(base).join("time_tracking")
}

/// Full path to the config file
fn config_path() -> PathBuf {
    data_dir().join(CONFIG_FILE)
}

// Legacy format for migration
#[derive(Deserialize)]
struct LegacyConfig {
    #[serde(default)]
    smartcard_active: bool,
}

// Task information including name and optional max hours
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct TaskInfo {
    pub(crate) name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) max_hours: Option<f64>,
}

impl TaskInfo {
    pub(crate) fn new(name: String) -> Self {
        Self {
            name,
            max_hours: None,
        }
    }

    pub(crate) fn with_max_hours(name: String, max_hours: Option<f64>) -> Self {
        Self { name, max_hours }
    }
}

// sectioned format
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub(crate) struct SmartcardConfig {
    /// Whether the smartcard auto-tracking feature is active
    #[serde(default)]
    pub(crate) active: bool,
}

/// Maps slot keys `"0"` - `"9"` to task information
pub(crate) type TaskMap = BTreeMap<String, TaskInfo>;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub(crate) struct TrackerConfig {
    #[serde(default)]
    pub(crate) smartcard: SmartcardConfig,

    /// Named task slots: key = "0" - "9", value = task name
    #[serde(default)]
    pub(crate) tasks: TaskMap,

    /// Maximum work hours per day (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) max_hours_per_day: Option<f64>,

    /// Display time in decimal format
    #[serde(default)]
    pub(crate) decimal_time_format: bool,
}

impl TrackerConfig {
    pub(crate) fn smartcard_active(&self) -> bool {
        self.smartcard.active
    }

    pub(crate) fn set_smartcard_active(&mut self, v: bool) {
        self.smartcard.active = v;
    }

    /// Return the task name for a slot (0 - 9), if configured
    pub(crate) fn task_name(&self, slot: u8) -> Option<&str> {
        self.tasks
            .get(&slot.to_string())
            .map(|info| info.name.as_str())
    }

    /// Get the full task info for a slot
    pub(crate) fn task_info(&self, slot: u8) -> Option<&TaskInfo> {
        self.tasks.get(&slot.to_string())
    }

    /// Set (or clear if `name` is empty) a task name for a slot
    /// Supports format "TaskName" or "TaskName:2.5" for name with max hours
    pub(crate) fn set_task_name(&mut self, slot: u8, name: &str) {
        let key = slot.to_string();
        if name.is_empty() {
            self.tasks.remove(&key);
        } else {
            // Parse "TaskName" or "TaskName:2.5"
            if let Some((task_name, hours_str)) = name.split_once(':') {
                let task_name = task_name.trim().to_owned();
                let max_hours = hours_str.trim().parse::<f64>().ok();
                self.tasks
                    .insert(key, TaskInfo::with_max_hours(task_name, max_hours));
            } else {
                self.tasks
                    .insert(key, TaskInfo::new(name.trim().to_owned()));
            }
        }
    }

    /// Get the maximum hours per day, if configured
    pub(crate) fn max_hours_per_day(&self) -> Option<f64> {
        self.max_hours_per_day
    }

    /// Set (or clear if None) the maximum hours per day
    pub(crate) fn set_max_hours_per_day(&mut self, hours: Option<f64>) {
        self.max_hours_per_day = hours;
    }

    /// Get the decimal time format preference
    pub(crate) fn decimal_time_format(&self) -> bool {
        self.decimal_time_format
    }

    /// Set the decimal time format preference
    pub(crate) fn set_decimal_time_format(&mut self, enabled: bool) {
        self.decimal_time_format = enabled;
    }

    /// Load config from disk and fall back to defaults when missing or invalid
    pub(crate) fn load() -> Self {
        let raw = match fs::read_to_string(config_path()) {
            Ok(s) => s,
            Err(_) => return Self::default(),
        };

        if let Ok(cfg) = toml::from_str::<TrackerConfig>(&raw)
            && (raw.contains("[smartcard]") || raw.contains("[tasks]"))
        {
            return cfg;
        }

        if let Ok(legacy) = toml::from_str::<LegacyConfig>(&raw) {
            let migrated = TrackerConfig {
                smartcard: SmartcardConfig {
                    active: legacy.smartcard_active,
                },
                tasks: BTreeMap::new(),
                max_hours_per_day: None,
                decimal_time_format: false,
            };
            let _ = migrated.save();
            return migrated;
        }

        Self::default()
    }

    /// Persist the configuration
    ///
    /// may returns error message for the TUI feedback line
    pub(crate) fn save(&self) -> Result<(), String> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| format!("Config dir: {err}"))?;
        }

        let serialized =
            toml::to_string_pretty(self).map_err(|err| format!("Config serialize: {err}"))?;

        fs::write(path, serialized).map_err(|err| format!("Config write: {err}"))
    }

    /// modify self
    #[allow(dead_code)]
    pub(crate) fn update<F>(&mut self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut TrackerConfig),
    {
        f(self);
        self.save()?;
        Ok(())
    }
}
