mod configuration;
mod editor;
mod fileio;
mod legacy;

pub(crate) use configuration::TrackerConfig;
use configuration::data_dir;
pub(crate) use editor::detect_editor;
pub(crate) use editor::open_in_editor;
pub(crate) use fileio::load_record;
pub(crate) use fileio::save_record;
pub(crate) use fileio::toml_path;

pub(crate) fn migrate_legacy_data() {
    legacy::migrate_all();
}
