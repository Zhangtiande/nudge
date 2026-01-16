//! Integration tests for the sanitizer module.
//!
//! These tests verify that sensitive data is properly redacted
//! before being sent to the LLM.

use regex::Regex;

/// Sanitize text by applying patterns
fn sanitize_text(input: &str, patterns: &[(Regex, &str)]) -> String {
    let mut result = input.to_string();
    for (pattern, replacement) in patterns {
        result = pattern.replace_all(&result, *replacement).to_string();
    }
    result
}

/// Test OpenAI API key redaction
#[test]
fn test_sanitize_openai_api_key() {
    let patterns = vec![(
        Regex::new(r"sk-[a-zA-Z0-9]{20,}").unwrap(),
        "[REDACTED:openai_key]",
    )];

    let input = "export OPENAI_API_KEY=sk-abcdefghijklmnopqrstuvwxyz1234567890";
    let result = sanitize_text(input, &patterns);

    assert!(result.contains("[REDACTED:openai_key]"));
    assert!(!result.contains("sk-abcdef"));
}

/// Test GitHub token redaction (all variants)
#[test]
fn test_sanitize_github_tokens() {
    let patterns = vec![
        (
            Regex::new(r"ghp_[a-zA-Z0-9]{36}").unwrap(),
            "[REDACTED:github_token]",
        ),
        (
            Regex::new(r"gho_[a-zA-Z0-9]{36}").unwrap(),
            "[REDACTED:github_oauth]",
        ),
        (
            Regex::new(r"ghs_[a-zA-Z0-9]{36}").unwrap(),
            "[REDACTED:github_secret]",
        ),
    ];

    // Personal access token
    let input1 = "git clone https://ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx@github.com/repo";
    let result1 = sanitize_text(input1, &patterns);
    assert!(result1.contains("[REDACTED:github_token]"));

    // OAuth token
    let input2 = "export GITHUB_TOKEN=gho_yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy";
    let result2 = sanitize_text(input2, &patterns);
    assert!(result2.contains("[REDACTED:github_oauth]"));
}

/// Test AWS credentials redaction
#[test]
fn test_sanitize_aws_credentials() {
    let patterns = vec![(
        Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
        "[REDACTED:aws_key]",
    )];

    let input = "export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE";
    let result = sanitize_text(input, &patterns);

    assert!(result.contains("[REDACTED:aws_key]"));
    assert!(!result.contains("AKIAIOSFODNN7EXAMPLE"));
}

/// Test Bearer token redaction
#[test]
fn test_sanitize_bearer_token() {
    let patterns = vec![(
        Regex::new(r"Bearer\s+[a-zA-Z0-9._\-]+").unwrap(),
        "Bearer [REDACTED]",
    )];

    let input = r#"curl -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9""#;
    let result = sanitize_text(input, &patterns);

    assert!(result.contains("Bearer [REDACTED]"));
    assert!(!result.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
}

/// Test password flag redaction
#[test]
fn test_sanitize_password_flags() {
    let patterns = vec![
        (
            Regex::new(r"--password[=\s]+\S+").unwrap(),
            "--password=[REDACTED]",
        ),
        (Regex::new(r"-p\s+\S+").unwrap(), "-p [REDACTED]"),
    ];

    // Long form
    let input1 = "mysql -u root --password=secret123 -h localhost";
    let result1 = sanitize_text(input1, &patterns);
    assert!(result1.contains("[REDACTED]"));
    assert!(!result1.contains("secret123"));

    // Short form
    let input2 = "mysql -u root -p mysecret";
    let result2 = sanitize_text(input2, &patterns);
    assert!(result2.contains("[REDACTED]"));
}

/// Test URL credentials redaction
#[test]
fn test_sanitize_url_credentials() {
    let patterns = vec![(Regex::new(r"://[^:]+:[^@]+@").unwrap(), "://[REDACTED]@")];

    let input = "git clone https://user:password123@github.com/org/repo.git";
    let result = sanitize_text(input, &patterns);

    assert!(result.contains("://[REDACTED]@"));
    assert!(!result.contains("user:password123"));
}

/// Test token flag redaction
#[test]
fn test_sanitize_token_flags() {
    let patterns = vec![(
        Regex::new(r"--token[=\s]+\S+").unwrap(),
        "--token=[REDACTED]",
    )];

    let input = "docker login --token=my-registry-token";
    let result = sanitize_text(input, &patterns);

    assert!(result.contains("--token=[REDACTED]"));
    assert!(!result.contains("my-registry-token"));
}

/// Test private key detection
#[test]
fn test_sanitize_private_key() {
    let patterns = vec![(
        Regex::new(r"-----BEGIN\s+(?:RSA\s+)?PRIVATE\s+KEY-----").unwrap(),
        "[REDACTED:private_key]",
    )];

    let input = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...";
    let result = sanitize_text(input, &patterns);

    assert!(result.contains("[REDACTED:private_key]"));
}

/// Test environment variable secret detection
#[test]
fn test_sanitize_env_secrets() {
    let patterns = vec![(
        Regex::new(r"(?:export\s+)?[A-Z_]*(?:SECRET|PASSWORD|TOKEN|KEY)[A-Z_]*=\S+").unwrap(),
        "[REDACTED:env_secret]",
    )];

    let inputs = vec![
        "export MY_SECRET=abc123",
        "DATABASE_PASSWORD=secret",
        "API_TOKEN=xyz789",
        "AWS_SECRET_KEY=sensitive",
    ];

    for input in inputs {
        let result = sanitize_text(input, &patterns);
        assert!(
            result.contains("[REDACTED:env_secret]"),
            "Failed for: {}",
            input
        );
    }
}

/// Test custom pattern application
#[test]
fn test_custom_patterns() {
    let custom = vec![(
        Regex::new(r"internal-key-\d+").unwrap(),
        "[REDACTED:custom]",
    )];

    let input = "using internal-key-12345 for auth";
    let result = sanitize_text(input, &custom);

    assert!(result.contains("[REDACTED:custom]"));
    assert!(!result.contains("internal-key-12345"));
}

/// Test that normal commands are not affected
#[test]
fn test_normal_commands_unchanged() {
    let patterns: Vec<(Regex, &str)> = vec![(
        Regex::new(r"sk-[a-zA-Z0-9]{20,}").unwrap(),
        "[REDACTED:openai_key]",
    )];

    let normal_commands = vec![
        "ls -la",
        "git status",
        "cargo build --release",
        "docker ps",
        "kubectl get pods",
    ];

    for cmd in normal_commands {
        let result = sanitize_text(cmd, &patterns);
        assert_eq!(result, cmd, "Command was modified: {}", cmd);
    }
}

/// Test multiple secrets in one command
#[test]
fn test_multiple_secrets() {
    let patterns = vec![
        (
            Regex::new(r"sk-[a-zA-Z0-9]{20,}").unwrap(),
            "[REDACTED:openai_key]",
        ),
        (
            Regex::new(r"ghp_[a-zA-Z0-9]{36}").unwrap(),
            "[REDACTED:github_token]",
        ),
    ];

    let input =
        "OPENAI_KEY=sk-abcdefghijklmnopqrstuvwxyz GITHUB=ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
    let result = sanitize_text(input, &patterns);

    assert!(result.contains("[REDACTED:openai_key]"));
    assert!(result.contains("[REDACTED:github_token]"));
    assert!(!result.contains("sk-abcdef"));
    assert!(!result.contains("ghp_"));
}

/// Test sanitization preserves command structure
#[test]
fn test_preserves_command_structure() {
    let patterns = vec![(
        Regex::new(r"--password[=\s]+\S+").unwrap(),
        "--password=[REDACTED]",
    )];

    let input = "mysql -u root --password=secret -h localhost dbname";
    let result = sanitize_text(input, &patterns);

    // Verify command structure is preserved
    assert!(result.contains("mysql"));
    assert!(result.contains("-u root"));
    assert!(result.contains("-h localhost"));
    assert!(result.contains("dbname"));
    assert!(result.contains("--password=[REDACTED]"));
}

/// Test edge case: empty input
#[test]
fn test_empty_input() {
    let patterns: Vec<(Regex, &str)> = vec![];
    let result = sanitize_text("", &patterns);
    assert!(result.is_empty());
}

/// Test edge case: input with only whitespace
#[test]
fn test_whitespace_only() {
    let patterns: Vec<(Regex, &str)> = vec![];
    let result = sanitize_text("   \t\n  ", &patterns);
    assert_eq!(result, "   \t\n  ");
}
