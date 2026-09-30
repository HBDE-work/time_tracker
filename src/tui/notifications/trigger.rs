use std::process::Command;

pub(crate) fn notify_max_time_exceeded() {
    #[cfg(target_os = "linux")]
    notify_linux();

    #[cfg(target_os = "windows")]
    notify_windows();
}

#[cfg(target_os = "linux")]
fn notify_linux() {
    let _ = Command::new("notify-send")
        .args([
            "Time tracker",
            "Maximum work time exceeded today!",
            "--urgency=critical",
        ])
        .spawn();
}

#[cfg(target_os = "windows")]
fn notify_windows() {
    // Windows desktop notifications need a proper toast/app-id integration
    //
    // Keep this as a no-op for now
}
