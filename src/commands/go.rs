use crate::data::EventKind;
use crate::tracking_logic::execute_action;

pub(crate) fn cmd_go() {
    println!("{}", execute_action(EventKind::Go));
}
