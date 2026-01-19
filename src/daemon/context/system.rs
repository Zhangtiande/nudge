// System information collection module

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sysinfo::System;
use tracing::debug;

/// System information context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system type: "Linux", "Windows", "Darwin" (macOS)
    pub os_type: String,
    /// Operating system version (e.g., "11 (Build 22621)" for Windows 11)
    pub os_version: String,
    /// System architecture (e.g., "x86_64", "aarch64")
    pub arch: String,
    /// Shell type (e.g., "bash", "zsh", "powershell", "cmd")
    pub shell_type: String,
    /// Current username
    pub username: String,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            os_type: String::from("unknown"),
            os_version: String::from("unknown"),
            arch: String::from("unknown"),
            shell_type: String::from("unknown"),
            username: String::from("unknown"),
        }
    }
}

/// Collect system information
pub fn collect_system_info(session_id: &str) -> Result<SystemInfo> {
    debug!("Collecting system information");

    // OS type (compile-time constant)
    let os_type = std::env::consts::OS.to_string();

    // OS version (runtime query)
    let os_version = System::os_version().unwrap_or_else(|| "unknown".to_string());

    // Architecture (compile-time constant)
    let arch = std::env::consts::ARCH.to_string();

    // Shell type (from session_id)
    let shell_type = detect_shell_type_from_session(session_id);

    // Username (environment variable)
    let username = get_username();

    debug!(
        "System info: OS={} {}, Arch={}, Shell={}, User={}",
        os_type, os_version, arch, shell_type, username
    );

    Ok(SystemInfo {
        os_type,
        os_version,
        arch,
        shell_type,
        username,
    })
}

/// Detect shell type from session_id
fn detect_shell_type_from_session(session_id: &str) -> String {
    if session_id.starts_with("bash-") {
        "bash".to_string()
    } else if session_id.starts_with("zsh-") {
        "zsh".to_string()
    } else if session_id.starts_with("pwsh-") || session_id.starts_with("powershell-") {
        "powershell".to_string()
    } else if session_id.starts_with("cmd-") {
        "cmd".to_string()
    } else {
        // Try to detect from environment
        #[cfg(unix)]
        {
            if let Ok(shell) = std::env::var("SHELL") {
                if shell.contains("zsh") {
                    return "zsh".to_string();
                } else if shell.contains("bash") {
                    return "bash".to_string();
                }
            }
            "bash".to_string()
        }

        #[cfg(windows)]
        {
            "powershell".to_string()
        }
    }
}

/// Get current username from environment variables
fn get_username() -> String {
    // Try different environment variables in order of preference
    #[cfg(windows)]
    {
        std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "unknown".to_string())
    }

    #[cfg(unix)]
    {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_shell_type_bash() {
        assert_eq!(detect_shell_type_from_session("bash-12345"), "bash");
    }

    #[test]
    fn test_detect_shell_type_zsh() {
        assert_eq!(detect_shell_type_from_session("zsh-54321"), "zsh");
    }

    #[test]
    fn test_detect_shell_type_powershell() {
        assert_eq!(detect_shell_type_from_session("pwsh-99999"), "powershell");
        assert_eq!(
            detect_shell_type_from_session("powershell-11111"),
            "powershell"
        );
    }

    #[test]
    fn test_detect_shell_type_cmd() {
        assert_eq!(detect_shell_type_from_session("cmd-88888"), "cmd");
    }

    #[test]
    fn test_collect_system_info() {
        let result = collect_system_info("bash-test");
        assert!(result.is_ok());

        let info = result.unwrap();
        assert!(!info.os_type.is_empty());
        assert!(!info.arch.is_empty());
        assert_eq!(info.shell_type, "bash");
    }

    #[test]
    fn test_get_username() {
        let username = get_username();
        assert!(!username.is_empty());
        assert_ne!(username, "unknown"); // Should get a real username in test environment
    }
}
