use lazy_static::lazy_static;
use regex::Regex;
use tracing::debug;

use super::context::ContextData;

/// Sanitization event for audit logging
#[derive(Debug, Clone)]
pub struct SanitizationEvent {
    pub pattern_type: String,
    pub original_length: usize,
}

lazy_static! {
    /// Built-in sensitive data patterns
    static ref SENSITIVE_PATTERNS: Vec<(Regex, &'static str)> = vec![
        // OpenAI API keys
        (Regex::new(r"sk-[a-zA-Z0-9]{20,}").unwrap(), "[REDACTED:openai_key]"),

        // GitHub tokens
        (Regex::new(r"ghp_[a-zA-Z0-9]{36}").unwrap(), "[REDACTED:github_token]"),
        (Regex::new(r"gho_[a-zA-Z0-9]{36}").unwrap(), "[REDACTED:github_oauth]"),
        (Regex::new(r"ghs_[a-zA-Z0-9]{36}").unwrap(), "[REDACTED:github_secret]"),

        // AWS credentials
        (Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(), "[REDACTED:aws_key]"),

        // Generic API keys (common patterns)
        (Regex::new(r#"api[_-]?key[=:\s]+['"]?[a-zA-Z0-9_-]{20,}['"]?"#).unwrap(), "api_key=[REDACTED]"),

        // Bearer tokens
        (Regex::new(r"Bearer\s+[a-zA-Z0-9._\-]+").unwrap(), "Bearer [REDACTED]"),

        // CLI passwords
        (Regex::new(r"--password[=\s]+\S+").unwrap(), "--password=[REDACTED]"),
        (Regex::new(r"-p\s+\S+").unwrap(), "-p [REDACTED]"),

        // CLI tokens
        (Regex::new(r"--token[=\s]+\S+").unwrap(), "--token=[REDACTED]"),

        // URL credentials (user:pass@host)
        (Regex::new(r"://[^:]+:[^@]+@").unwrap(), "://[REDACTED]@"),

        // Private keys (PEM format start)
        (Regex::new(r"-----BEGIN\s+(?:RSA\s+)?PRIVATE\s+KEY-----").unwrap(), "[REDACTED:private_key]"),

        // Environment variable assignments with secrets
        (Regex::new(r"(?:export\s+)?[A-Z_]*(?:SECRET|PASSWORD|TOKEN|KEY)[A-Z_]*=\S+").unwrap(), "[REDACTED:env_secret]"),
    ];
}

/// Sanitize context data
pub fn sanitize(
    context: &ContextData,
    custom_patterns: &[String],
) -> (ContextData, Vec<SanitizationEvent>) {
    let mut result = context.clone();
    let mut events = Vec::new();

    // Compile custom patterns
    let custom_regexes: Vec<Regex> = custom_patterns
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect();

    // Sanitize history
    for cmd in &mut result.history {
        let (sanitized, cmd_events) = sanitize_text(cmd, &custom_regexes);
        *cmd = sanitized;
        events.extend(cmd_events);
    }

    // Sanitize git commit messages
    if let Some(ref mut git) = result.git {
        for commit in &mut git.recent_commits {
            let (sanitized, commit_events) = sanitize_text(commit, &custom_regexes);
            *commit = sanitized;
            events.extend(commit_events);
        }
    }

    if !events.is_empty() {
        debug!("Sanitized {} sensitive items", events.len());
    }

    (result, events)
}

/// Sanitize a single text string
fn sanitize_text(input: &str, custom_patterns: &[Regex]) -> (String, Vec<SanitizationEvent>) {
    let mut result = input.to_string();
    let mut events = Vec::new();

    // Apply built-in patterns
    for (pattern, replacement) in SENSITIVE_PATTERNS.iter() {
        if pattern.is_match(&result) {
            for mat in pattern.find_iter(&result.clone()) {
                events.push(SanitizationEvent {
                    pattern_type: replacement.to_string(),
                    original_length: mat.as_str().len(),
                });
            }
            result = pattern.replace_all(&result, *replacement).to_string();
        }
    }

    // Apply custom patterns
    for pattern in custom_patterns {
        if pattern.is_match(&result) {
            for mat in pattern.find_iter(&result.clone()) {
                events.push(SanitizationEvent {
                    pattern_type: "[REDACTED:custom]".to_string(),
                    original_length: mat.as_str().len(),
                });
            }
            result = pattern
                .replace_all(&result, "[REDACTED:custom]")
                .to_string();
        }
    }

    (result, events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_openai_key() {
        let (result, events) =
            sanitize_text("export OPENAI_API_KEY=sk-abcdef1234567890abcdefghij", &[]);
        assert!(result.contains("[REDACTED"));
        assert!(!result.contains("sk-abcdef"));
        assert!(!events.is_empty());
    }

    #[test]
    fn test_sanitize_github_token() {
        let (result, _) = sanitize_text(
            "git clone https://ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx@github.com/repo",
            &[],
        );
        assert!(result.contains("[REDACTED"));
        assert!(!result.contains("ghp_"));
    }

    #[test]
    fn test_sanitize_bearer_token() {
        let (result, _) = sanitize_text(
            "curl -H 'Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9'",
            &[],
        );
        assert!(result.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_password_flag() {
        let (result, _) = sanitize_text("mysql -u root --password=secret123", &[]);
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("secret123"));
    }

    #[test]
    fn test_sanitize_url_credentials() {
        let (result, _) = sanitize_text("git clone https://user:pass@github.com/repo", &[]);
        assert!(result.contains("[REDACTED]@"));
        assert!(!result.contains("user:pass"));
    }

    #[test]
    fn test_custom_pattern() {
        let custom = vec![Regex::new(r"my-secret-\d+").unwrap()];
        let (result, events) = sanitize_text("using my-secret-12345 here", &custom);
        assert!(result.contains("[REDACTED:custom]"));
        assert!(!result.contains("my-secret-12345"));
        assert!(!events.is_empty());
    }
}
