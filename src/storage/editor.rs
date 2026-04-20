use std::env;
use std::path::Path;
use std::process::Command;

/// Detect available text editor on the system
pub(crate) fn detect_editor() -> Option<String> {
    // Check environment variables
    if let Ok(editor) = env::var("EDITOR")
        && !editor.is_empty()
        && command_exists(&editor)
    {
        return Some(editor);
    }

    if let Ok(visual) = env::var("VISUAL")
        && !visual.is_empty()
        && command_exists(&visual)
    {
        return Some(visual);
    }

    // Platform-specific fallbacks
    if cfg!(target_os = "windows") {
        // Windows fallbacks
        let windows_editors = vec!["notepad.exe", "notepad++.exe", "code.cmd"];
        for editor in windows_editors {
            if command_exists(editor) {
                return Some(editor.to_string());
            }
        }
    } else {
        // Unix/Linux/macOS fallbacks
        let unix_editors = vec!["vim", "nano", "vi", "emacs", "gedit", "kate"];
        for editor in unix_editors {
            if command_exists(editor) {
                return Some(editor.to_string());
            }
        }
    }

    None
}

/// Check if a command exists in PATH
fn command_exists(cmd: &str) -> bool {
    let cmd_name = cmd.split_whitespace().next().unwrap_or(cmd);

    if cfg!(target_os = "windows") {
        // try 'where' command
        Command::new("where")
            .arg(cmd_name)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    } else {
        // try 'which' command
        Command::new("which")
            .arg(cmd_name)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// Open a file in the detected editor
/// Returns Ok(()) if editor was launched successfully, Err with message otherwise
pub(crate) fn open_in_editor(file_path: &Path) -> Result<(), String> {
    let editor = detect_editor()
        .ok_or_else(|| "No text editor found. Set EDITOR environment variable.".to_string())?;

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", file_path.display()));
    }

    // Split editor command in case it includes arguments
    let mut parts = editor.split_whitespace();
    let editor_cmd = parts.next().unwrap();
    let editor_args: Vec<&str> = parts.collect();

    let mut cmd = Command::new(editor_cmd);

    for arg in editor_args {
        cmd.arg(arg);
    }

    cmd.arg(file_path);

    match cmd.status() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(format!("Editor exited with status: {}", status))
            }
        }
        Err(err) => Err(format!("Failed to launch editor '{}': {}", editor, err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_editor() {
        let editor = detect_editor();
        // Should find at least one editor on any system
        assert!(editor.is_some(), "Should detect at least one editor");
    }

    #[test]
    fn test_command_exists() {
        // Test with a command that should always exist
        #[cfg(target_os = "windows")]
        assert!(command_exists("cmd.exe"));

        #[cfg(not(target_os = "windows"))]
        assert!(command_exists("ls"));
    }
}
