use crate::data::EventKind;
use crate::tracking_logic::execute_action;

pub fn cmd_pause() {
    println!("{}", execute_action(EventKind::Pause));
}
