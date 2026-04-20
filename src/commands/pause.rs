use crate::commands::stop_task_then;
use crate::data::EventKind;

pub(crate) fn cmd_pause() {
    stop_task_then(EventKind::Pause)
}
