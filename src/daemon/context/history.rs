use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::UserDirs;
use tracing::debug;

/// Read shell history
pub fn read_history(session_id: &str, window_size: usize) -> Result<Vec<String>> {
    let history_path = get_history_path(session_id)?;

    if !history_path.exists() {
        debug!("History file not found: {}", history_path.display());
        return Ok(Vec::new());
    }

    // Gracefully handle read errors - use lossy UTF-8 conversion for non-UTF-8 content
    let bytes = match fs::read(&history_path) {
        Ok(b) => b,
        Err(e) => {
            debug!("Cannot read history file {}: {} (continuing without history)", history_path.display(), e);
            return Ok(Vec::new());
        }
    };
    // Use lossy conversion to handle non-UTF-8 bytes (common in zsh history)
    let contents = String::from_utf8_lossy(&bytes).into_owned();

    let shell_type = detect_shell_type(session_id);
    let entries = parse_history(&contents, shell_type);

    // Deduplicate consecutive commands and limit to window size
    let deduplicated = deduplicate(entries);
    let limited: Vec<String> = deduplicated.into_iter().rev().take(window_size).rev().collect();

    Ok(limited)
}

/// Detect shell type from session ID
fn detect_shell_type(session_id: &str) -> ShellType {
    if session_id.starts_with("bash-") {
        ShellType::Bash
    } else if session_id.starts_with("zsh-") {
        ShellType::Zsh
    } else if session_id.starts_with("pwsh-") || session_id.starts_with("powershell-") {
        ShellType::PowerShell
    } else if session_id.starts_with("cmd-") {
        ShellType::Cmd
    } else {
        // Try to detect from environment
        #[cfg(unix)]
        {
            if let Ok(shell) = std::env::var("SHELL") {
                if shell.contains("zsh") {
                    return ShellType::Zsh;
                }
                return ShellType::Bash;
            }
            ShellType::Bash
        }

        #[cfg(windows)]
        {
            // On Windows, default to PowerShell (most common)
            ShellType::PowerShell
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ShellType {
    Bash,
    Zsh,
    PowerShell,
    Cmd,
}

/// Get the history file path
fn get_history_path(session_id: &str) -> Result<PathBuf> {
    let user_dirs = UserDirs::new().context("Failed to get user directories")?;
    let home = user_dirs.home_dir();

    let shell_type = detect_shell_type(session_id);

    let path = match shell_type {
        ShellType::Bash => home.join(".bash_history"),
        ShellType::Zsh => {
            // Check for ZDOTDIR first
            if let Ok(zdotdir) = std::env::var("ZDOTDIR") {
                PathBuf::from(zdotdir).join(".zsh_history")
            } else {
                home.join(".zsh_history")
            }
        }
        ShellType::PowerShell => {
            // PowerShell PSReadLine history path
            // Typical path: %APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
            #[cfg(windows)]
            {
                let appdata = std::env::var("APPDATA")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| home.join("AppData").join("Roaming"));
                appdata
                    .join("Microsoft")
                    .join("Windows")
                    .join("PowerShell")
                    .join("PSReadLine")
                    .join("ConsoleHost_history.txt")
            }
            #[cfg(unix)]
            {
                // PowerShell on Unix uses different path
                // ~/.local/share/powershell/PSReadLine/ConsoleHost_history.txt
                let local_share = if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
                    PathBuf::from(xdg_data)
                } else {
                    home.join(".local").join("share")
                };
                local_share
                    .join("powershell")
                    .join("PSReadLine")
                    .join("ConsoleHost_history.txt")
            }
        }
        ShellType::Cmd => {
            // CMD doesn't maintain a persistent history file
            anyhow::bail!("CMD does not maintain a persistent history file");
        }
    };

    Ok(path)
}

/// Parse history file contents
fn parse_history(contents: &str, shell_type: ShellType) -> Vec<String> {
    match shell_type {
        ShellType::Bash => parse_bash_history(contents),
        ShellType::Zsh => parse_zsh_history(contents),
        ShellType::PowerShell => parse_powershell_history(contents),
        ShellType::Cmd => Vec::new(), // CMD has no history file
    }
}

/// Parse bash history (simple line-by-line)
fn parse_bash_history(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect()
}

/// Parse zsh history (handles extended format with timestamps)
fn parse_zsh_history(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| {
            if line.is_empty() {
                return None;
            }

            // Zsh extended history format: : timestamp:duration;command
            if line.starts_with(": ") {
                // Find the semicolon that separates metadata from command
                if let Some(idx) = line.find(';') {
                    let command = &line[idx + 1..];
                    if !command.is_empty() {
                        return Some(command.to_string());
                    }
                }
                None
            } else {
                // Simple format (no timestamps)
                Some(line.to_string())
            }
        })
        .collect()
}

/// Parse PowerShell history (simple line-by-line format like Bash)
fn parse_powershell_history(contents: &str) -> Vec<String> {
    // PowerShell PSReadLine history is stored as simple lines
    contents
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect()
}

/// Deduplicate consecutive identical commands
fn deduplicate(entries: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    let mut last: Option<String> = None;

    for entry in entries {
        if last.as_ref() != Some(&entry) {
            result.push(entry.clone());
            last = Some(entry);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bash_history() {
        let history = "ls -la\ncd /home\nls -la\ngit status\n";
        let entries = parse_bash_history(history);
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0], "ls -la");
        assert_eq!(entries[2], "ls -la");
    }

    #[test]
    fn test_parse_zsh_history() {
        let history = ": 1705123456:0;ls -la\n: 1705123457:0;cd /home\n: 1705123458:0;git status\n";
        let entries = parse_zsh_history(history);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0], "ls -la");
        assert_eq!(entries[1], "cd /home");
    }

    #[test]
    fn test_parse_powershell_history() {
        let history = "Get-Process\nGet-Service\nls\ncd C:\\Users\n";
        let entries = parse_powershell_history(history);
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0], "Get-Process");
        assert_eq!(entries[3], "cd C:\\Users");
    }

    #[test]
    fn test_deduplicate() {
        let entries = vec![
            "ls".to_string(),
            "ls".to_string(),
            "cd".to_string(),
            "ls".to_string(),
        ];
        let deduped = deduplicate(entries);
        assert_eq!(deduped, vec!["ls", "cd", "ls"]);
    }
}
