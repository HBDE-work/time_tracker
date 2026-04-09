use chrono::NaiveDate;
use std::fs;
use std::path::PathBuf;

use crate::data::DayRecord;

use super::data_dir;

fn file_for_date(date: NaiveDate) -> PathBuf {
    data_dir().join(format!("{}.json", date))
}

pub(crate) fn load_record(date: NaiveDate) -> Option<DayRecord> {
    let path = file_for_date(date);
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub(crate) fn save_record(record: &DayRecord) {
    let dir = data_dir();
    fs::create_dir_all(&dir).expect("Failed to create data directory");
    let path = file_for_date(record.date);
    let json = serde_json::to_string_pretty(record).expect("Failed to serialize record");
    fs::write(path, json).expect("Failed to write record file");
}
