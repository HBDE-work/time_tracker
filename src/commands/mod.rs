mod go;
mod pause;
mod status;
mod stop;

use crate::data::EventKind;
use crate::tracking_logic::execute_action;
use crate::tracking_logic::stop_active_task;

pub(crate) use go::cmd_go;
pub(crate) use pause::cmd_pause;
pub(crate) use status::cmd_status;
pub(crate) use stop::cmd_stop;

pub(crate) fn stop_task_then(action: EventKind) {
    if let Some(message) = stop_active_task() {
        println!("{message}");
    }
    println!("{}", execute_action(action));
}
