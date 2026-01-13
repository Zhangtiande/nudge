use lazy_static::lazy_static;
use regex::Regex;
use tracing::debug;

use crate::protocol::Warning;

lazy_static! {
    /// Built-in dangerous command patterns
    static ref DANGEROUS_PATTERNS: Vec<(Regex, &'static str)> = vec![
        // Recursive deletion of root or home
        (Regex::new(r"rm\s+(-[rfRF]+\s+)*(/|~|\$HOME)\s*$").unwrap(),
         "This command will recursively delete the root/home directory"),
        (Regex::new(r"rm\s+(-[rfRF]+\s+)+\*\s*$").unwrap(),
         "This command will recursively delete all files"),
        (Regex::new(r"rm\s+-rf\s+/\s*$").unwrap(),
         "This command will destroy your system"),

        // Disk formatting
        (Regex::new(r"mkfs\.\w+\s+").unwrap(),
         "This command will format a disk, destroying all data"),
        (Regex::new(r"dd\s+if=.*of=/dev/(?:sd|nvme|hd)").unwrap(),
         "This command may overwrite disk data"),

        // Fork bomb
        (Regex::new(r":\(\)\s*\{\s*:\|:&\s*\}").unwrap(),
         "This is a fork bomb that will crash your system"),

        // Chmod dangerous permissions
        (Regex::new(r"chmod\s+(-R\s+)?777\s+/").unwrap(),
         "Setting 777 permissions on root is a security risk"),

        // Dangerous curl | bash pattern
        (Regex::new(r"curl\s+.*\|\s*(ba)?sh").unwrap(),
         "Piping untrusted content to shell is dangerous"),

        // Overwriting important files
        (Regex::new(r">\s*/etc/passwd").unwrap(),
         "This will destroy the password file"),
        (Regex::new(r">\s*/etc/shadow").unwrap(),
         "This will destroy the shadow password file"),

        // Kill all processes
        (Regex::new(r"kill\s+-9\s+-1").unwrap(),
         "This will kill all processes"),
        (Regex::new(r"pkill\s+-9\s+.").unwrap(),
         "This may kill important processes"),
    ];
}

/// Check if a command is potentially dangerous
pub fn check(command: &str, custom_patterns: &[String]) -> Option<Warning> {
    // Check built-in patterns
    for (pattern, message) in DANGEROUS_PATTERNS.iter() {
        if pattern.is_match(command) {
            debug!("Dangerous command detected: {}", command);
            return Some(Warning::dangerous(*message));
        }
    }

    // Check custom patterns
    for pattern_str in custom_patterns {
        if let Ok(pattern) = Regex::new(pattern_str) {
            if pattern.is_match(command) {
                debug!("Custom dangerous pattern matched: {}", command);
                return Some(Warning::dangerous("This command matches a custom dangerous pattern"));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rm_rf_root() {
        let warning = check("rm -rf /", &[]);
        assert!(warning.is_some());
    }

    #[test]
    fn test_detect_rm_rf_wildcard() {
        let warning = check("rm -rf *", &[]);
        assert!(warning.is_some());
    }

    #[test]
    fn test_detect_mkfs() {
        let warning = check("mkfs.ext4 /dev/sda1", &[]);
        assert!(warning.is_some());
    }

    #[test]
    fn test_detect_dd() {
        let warning = check("dd if=/dev/zero of=/dev/sda", &[]);
        assert!(warning.is_some());
    }

    #[test]
    fn test_safe_command() {
        let warning = check("ls -la", &[]);
        assert!(warning.is_none());
    }

    #[test]
    fn test_safe_rm() {
        let warning = check("rm -rf ./build", &[]);
        assert!(warning.is_none());
    }

    #[test]
    fn test_custom_pattern() {
        let custom = vec![r"dangerous-script".to_string()];
        let warning = check("./dangerous-script.sh", &custom);
        assert!(warning.is_some());
    }
}
