use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = ".tracker_options";

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
pub(crate) fn config_path() -> PathBuf {
    data_dir().join(CONFIG_FILE)
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub(crate) struct TrackerConfig {
    /// Whether the smartcard auto-tracking feature is active
    pub smartcard_active: bool,
}

impl TrackerConfig {
    /// Load the configuration
    ///
    /// if the config can't be loaded the default is returned
    pub fn load() -> Self {
        let path = config_path();
        if let Ok(mut file) = File::open(&path) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok()
                && let Ok(cfg) = toml::from_str(&contents)
            {
                return cfg;
            }
        }

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        TrackerConfig::default()
    }

    /// Persist the configuration
    pub fn save(&self) {
        let path = config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let serialized = toml::to_string_pretty(self).unwrap_or_else(|_| String::new());

        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
        {
            let _ = file.write_all(serialized.as_bytes());
        }
    }

    /// modify self
    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(&mut TrackerConfig),
    {
        f(self);
        self.save()
    }
}
