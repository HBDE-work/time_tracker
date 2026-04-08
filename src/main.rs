mod arguments;
mod commands;
mod data;
mod tracking_logic;
mod tui;

use arguments::{Args, Commands};
use clap::Parser;

fn main() {
    let args = Args::parse();

    match args.command {
        Some(Commands::Go) => commands::cmd_go(),
        Some(Commands::Pause) => commands::cmd_pause(),
        Some(Commands::Stop) => commands::cmd_stop(),
        Some(Commands::Status { day, week, year }) => commands::cmd_status(day, week, year),
        Some(Commands::Tui) => run_tui_here(),
        None => {
            if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
                run_tui_here();
            } else {
                spawn_terminal_with_tui();
            }
        }
    }
}

fn run_tui_here() {
    if let Err(err) = tui::run_tui() {
        eprintln!("TUI error: {err}");
    }
}

/// Spawns a terminal emulator running `tracker tui` inside it
fn spawn_terminal_with_tui() {
    let exe = std::env::current_exe().expect("Failed to resolve own executable path");

    #[cfg(target_os = "linux")]
    let result = try_spawn_linux(&exe);

    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd")
        .args(["/C", "start", "", &exe.display().to_string(), "tui"])
        .spawn();

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    let result: Result<std::process::Child, std::io::Error> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "unsupported OS",
    ));

    match result {
        Ok(_) => {}
        Err(err) => eprintln!("Failed to spawn terminal: {err}"),
    }
}

/// Tries common Linux terminal emulators in order of preference
#[cfg(target_os = "linux")]
fn try_spawn_linux(exe: &std::path::Path) -> Result<std::process::Child, std::io::Error> {
    let exe_str = exe.display().to_string();
    let emulators: &[(&str, &[&str])] = &[
        ("kitty", &[&exe_str, "tui"]),
        ("alacritty", &["-e", &exe_str, "tui"]),
        ("gnome-terminal", &["--", &exe_str, "tui"]),
        ("foot", &["--", &exe_str, "tui"]),
        ("wezterm", &["start", "--", &exe_str, "tui"]),
        ("xterm", &["-e", &exe_str, "tui"]),
        ("x-terminal-emulator", &["-e", &exe_str, "tui"]),
    ];

    for (name, args) in emulators {
        match std::process::Command::new(name).args(*args).spawn() {
            Ok(child) => return Ok(child),
            Err(_) => continue,
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No terminal emulator found. Install one or run 'tracker tui' from a shell.",
    ))
}
