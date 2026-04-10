use crate::data::EventKind;
use crate::tracking_logic::{execute_action, stop_active_task};

pub(crate) fn cmd_stop() {
    if let Some(msg) = stop_active_task() {
        println!("{msg}");
    }
    println!("{}", execute_action(EventKind::Stop));
}
