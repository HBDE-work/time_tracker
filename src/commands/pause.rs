use crate::data::EventKind;
use crate::tracking_logic::execute_action;

pub(crate) fn cmd_pause() {
    println!("{}", execute_action(EventKind::Pause));
}
