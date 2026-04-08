use crate::data::EventKind;
use crate::tracking_logic::execute_action;

pub fn cmd_go() {
    println!("{}", execute_action(EventKind::Go));
}
