use chrono::NaiveDate;
use std::fs;
use std::path::PathBuf;

use crate::data::DayRecord;

use super::data_dir;
use super::legacy;

pub(crate) fn toml_path(date: NaiveDate) -> PathBuf {
    data_dir().join(format!("{date}.toml"))
}

fn json_path(date: NaiveDate) -> PathBuf {
    data_dir().join(format!("{date}.json"))
}

pub(crate) fn load_record(date: NaiveDate) -> Option<DayRecord> {
    let toml_file = toml_path(date);

    if let Ok(content) = fs::read_to_string(&toml_file) {
        return toml::from_str(&content).ok();
    }

    // Try migrating a legacy JSON file
    let json_file = json_path(date);
    let json_content = fs::read_to_string(&json_file).ok()?;
    let migrated = legacy::migrate_json(&json_content)?;

    // Persist the migrated record as TOML
    save_record(&migrated);

    // Remove the old JSON file after successful migration
    let _ = fs::remove_file(&json_file);

    Some(migrated)
}

pub(crate) fn save_record(record: &DayRecord) {
    let dir = data_dir();
    fs::create_dir_all(&dir).expect("Failed to create data directory");
    let path = toml_path(record.date);
    let serialized = toml::to_string_pretty(record).expect("Failed to serialize record");
    fs::write(path, serialized).expect("Failed to write record file");
}
