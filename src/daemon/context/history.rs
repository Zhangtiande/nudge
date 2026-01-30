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
            debug!(
                "Cannot read history file {}: {} (continuing without history)",
                history_path.display(),
                e
            );
            return Ok(Vec::new());
        }
    };
    // Use lossy conversion to handle non-UTF-8 bytes (common in zsh history)
    let contents = String::from_utf8_lossy(&bytes).into_owned();

    let shell_type = detect_shell_type(session_id);
    let entries = parse_history(&contents, shell_type);

    // Deduplicate consecutive commands and limit to window size
    let deduplicated = deduplicate(entries);
    let limited: Vec<String> = deduplicated
        .into_iter()
        .rev()
        .take(window_size)
        .rev()
        .collect();

    Ok(limited)
}

/// Read recent history without session context (for diagnosis)
/// Auto-detects shell from environment
pub fn read_recent(count: usize) -> Result<Vec<String>> {
    // Try to auto-detect shell from environment
    let session_id = if cfg!(unix) {
        if let Ok(shell) = std::env::var("SHELL") {
            if shell.contains("zsh") {
                "zsh-auto"
            } else {
                "bash-auto"
            }
        } else {
            "bash-auto"
        }
    } else {
        "pwsh-auto"
    };

    read_history(session_id, count)
}

/// Find similar commands from history based on query string
pub fn find_similar_commands(
    session_id: &str,
    query: &str,
    window_size: usize,
    max_results: usize,
) -> Result<Vec<String>> {
    // Read history with larger window for searching
    let history_path = get_history_path(session_id)?;

    if !history_path.exists() {
        debug!("History file not found: {}", history_path.display());
        return Ok(Vec::new());
    }

    let bytes = match fs::read(&history_path) {
        Ok(b) => b,
        Err(e) => {
            debug!(
                "Cannot read history file {}: {} (continuing without similar commands)",
                history_path.display(),
                e
            );
            return Ok(Vec::new());
        }
    };

    let contents = String::from_utf8_lossy(&bytes).into_owned();
    let shell_type = detect_shell_type(session_id);
    let entries = parse_history(&contents, shell_type);

    // Extract keywords from query (ignore common shell keywords)
    let keywords = extract_keywords(query);
    if keywords.is_empty() {
        return Ok(Vec::new());
    }

    // Filter commands that contain any of the keywords (case-insensitive)
    let mut similar_commands: Vec<String> = entries
        .into_iter()
        .rev() // Start from most recent
        .take(window_size) // Limit search window
        .filter(|cmd| {
            let cmd_lower = cmd.to_lowercase();
            keywords
                .iter()
                .any(|keyword| cmd_lower.contains(&keyword.to_lowercase()))
        })
        .collect();

    // Remove consecutive duplicates
    similar_commands = deduplicate(similar_commands);

    // Limit to max_results
    similar_commands.truncate(max_results);

    debug!(
        "Found {} similar commands for query: {}",
        similar_commands.len(),
        query
    );

    Ok(similar_commands)
}

/// Extract keywords from query string, filtering out common shell commands
fn extract_keywords(query: &str) -> Vec<String> {
    const COMMON_COMMANDS: &[&str] = &[
        "cd", "ls", "pwd", "echo", "cat", "grep", "sed", "awk", "rm", "mv", "cp", "mkdir", "touch",
        "chmod", "chown", "sudo", "su", "exit", "clear", "history",
    ];

    query
        .split_whitespace()
        .filter(|word| !word.is_empty() && word.len() >= 2) // At least 2 characters
        .filter(|word| !COMMON_COMMANDS.contains(word))
        .map(|word| word.to_string())
        .collect()
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

    #[test]
    fn test_extract_keywords() {
        let query = "docker ps -a";
        let keywords = extract_keywords(query);
        assert_eq!(keywords, vec!["docker", "ps", "-a"]);

        let query2 = "cd ls"; // Common commands should be filtered
        let keywords2 = extract_keywords(query2);
        assert_eq!(keywords2.len(), 0);

        let query3 = "git commit -m message";
        let keywords3 = extract_keywords(query3);
        assert!(keywords3.contains(&"git".to_string()));
        assert!(keywords3.contains(&"commit".to_string()));
    }

    #[test]
    fn test_extract_keywords_min_length() {
        let query = "a b cd docker";
        let keywords = extract_keywords(query);
        // "a" and "b" are too short, "cd" is filtered as common command
        assert_eq!(keywords, vec!["docker"]);
    }
}
