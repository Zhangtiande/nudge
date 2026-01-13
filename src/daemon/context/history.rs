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

    // Gracefully handle permission errors - return empty history instead of failing
    let contents = match fs::read_to_string(&history_path) {
        Ok(c) => c,
        Err(e) => {
            debug!("Cannot read history file {}: {} (continuing without history)", history_path.display(), e);
            return Ok(Vec::new());
        }
    };

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
    } else {
        // Try to detect from SHELL env var
        if let Ok(shell) = std::env::var("SHELL") {
            if shell.contains("zsh") {
                return ShellType::Zsh;
            }
        }
        ShellType::Bash
    }
}

#[derive(Debug, Clone, Copy)]
enum ShellType {
    Bash,
    Zsh,
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
    };

    Ok(path)
}

/// Parse history file contents
fn parse_history(contents: &str, shell_type: ShellType) -> Vec<String> {
    match shell_type {
        ShellType::Bash => parse_bash_history(contents),
        ShellType::Zsh => parse_zsh_history(contents),
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
