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

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub(crate) struct TrackerConfig {
    /// Whether the smartcard auto-tracking feature is active
    pub(crate) smartcard_active: bool,
}

impl TrackerConfig {
    /// Load the configuration
    ///
    /// Returns the default if the file is missing or unreadable
    pub(crate) fn load() -> Self {
        fs::read_to_string(config_path())
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
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
