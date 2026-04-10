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

// sectioned format
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub(crate) struct SmartcardConfig {
    /// Whether the smartcard auto-tracking feature is active
    #[serde(default)]
    pub(crate) active: bool,
}

/// Maps slot keys `"0"` - `"9"` to task names
pub(crate) type TaskMap = BTreeMap<String, String>;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub(crate) struct TrackerConfig {
    #[serde(default)]
    pub(crate) smartcard: SmartcardConfig,

    /// Named task slots: key = "0" - "9", value = task name
    #[serde(default)]
    pub(crate) tasks: TaskMap,
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
        self.tasks.get(&slot.to_string()).map(|s| s.as_str())
    }

    /// Set (or clear if `name` is empty) a task name for a slot
    pub(crate) fn set_task_name(&mut self, slot: u8, name: &str) {
        let key = slot.to_string();
        if name.is_empty() {
            self.tasks.remove(&key);
        } else {
            self.tasks.insert(key, name.to_owned());
        }
    }

    /// Load the configuration
    ///
    /// Transparently migrates the old flat format (`smartcard_active = true`)
    /// into the new sectioned format and re-saves
    ///
    /// Returns the default if the file is missing or unreadable
    pub(crate) fn load() -> Self {
        let raw = match fs::read_to_string(config_path()) {
            Ok(s) => s,
            Err(_) => return Self::default(),
        };

        // Try the current sectioned format first
        if let Ok(cfg) = toml::from_str::<TrackerConfig>(&raw) {
            // if the raw text contains a `[smartcard]` table we it's the new format
            if raw.contains("[smartcard]") || raw.contains("[tasks]") {
                return cfg;
            }
        }

        // Fall back to the legacy flat format and migrate
        if let Ok(legacy) = toml::from_str::<LegacyConfig>(&raw) {
            let migrated = TrackerConfig {
                smartcard: SmartcardConfig {
                    active: legacy.smartcard_active,
                },
                tasks: BTreeMap::new(),
            };
            // Best-effort migration write
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
