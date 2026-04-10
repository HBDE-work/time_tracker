use crate::data::EventKind;
use crate::tracking_logic::{execute_action, start_task, stop_active_task};

pub(crate) fn cmd_go(task: Option<String>) {
    match task {
        Some(name) => {
            // Ensure global tracking is running first
            let go_msg = execute_action(EventKind::Go);
            let task_msg = start_task(&name);
            println!("{go_msg}");
            println!("{task_msg}");
        }
        None => {
            // return to global time (close any open task)
            if let Some(msg) = stop_active_task() {
                println!("{msg}");
            }
            println!("{}", execute_action(EventKind::Go));
        }
    }
}
