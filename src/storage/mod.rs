mod configuration;
mod fileio;
mod legacy;

pub(crate) use configuration::TrackerConfig;
use configuration::data_dir;
pub(crate) use fileio::load_record;
pub(crate) use fileio::save_record;

pub(crate) fn migrate_legacy_data() {
    legacy::migrate_all();
}
