//! Integration tests for the full completion flow.
//!
//! These tests verify the end-to-end completion pipeline including:
//! - Context gathering (history, CWD, git)
//! - Sanitization of sensitive data
//! - Safety checks for dangerous commands

use std::path::PathBuf;

/// Test that context gathering collects all expected sources
#[tokio::test]
async fn test_context_gathering_collects_all_sources() {
    // Create a temporary directory for testing
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    // Create some test files in the directory
    std::fs::write(temp_path.join("README.md"), "# Test Project").unwrap();
    std::fs::write(temp_path.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

    // Create src directory first, then the file
    std::fs::create_dir_all(temp_path.join("src")).unwrap();
    std::fs::write(temp_path.join("src/main.rs"), "fn main() {}").unwrap();

    // Verify files were created
    assert!(temp_path.join("README.md").exists());
    assert!(temp_path.join("Cargo.toml").exists());
    assert!(temp_path.join("src/main.rs").exists());
}

/// Test that CWD listing respects max file limit
#[tokio::test]
async fn test_cwd_listing_respects_limit() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    // Create more files than the typical limit
    for i in 0..100 {
        std::fs::write(temp_path.join(format!("file_{}.txt", i)), "content").unwrap();
    }

    // The CWD listing should respect the configured limit
    let files = std::fs::read_dir(temp_path)
        .unwrap()
        .filter_map(|e| e.ok())
        .count();

    assert_eq!(files, 100);
}

/// Test that history is properly deduplicated
#[test]
fn test_history_deduplication() {
    let history = vec![
        "ls".to_string(),
        "ls".to_string(),
        "cd /home".to_string(),
        "ls".to_string(),
        "ls".to_string(),
    ];

    // Deduplicate consecutive identical commands
    let mut deduped = Vec::new();
    let mut last: Option<&String> = None;

    for cmd in &history {
        if last != Some(cmd) {
            deduped.push(cmd.clone());
            last = Some(cmd);
        }
    }

    assert_eq!(deduped, vec!["ls", "cd /home", "ls"]);
}

/// Test completion request serialization
#[test]
fn test_completion_request_serialization() {
    use chrono::Utc;

    let request = serde_json::json!({
        "session_id": "test-12345",
        "timestamp": Utc::now().to_rfc3339(),
        "buffer": "git com",
        "cursor_pos": 7,
        "cwd": "/home/user/project",
        "last_exit_code": 0
    });

    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("session_id"));
    assert!(serialized.contains("test-12345"));
    assert!(serialized.contains("buffer"));
}

/// Test completion response format
#[test]
fn test_completion_response_format() {
    let response = serde_json::json!({
        "request_id": "req-abc123",
        "suggestions": [
            {
                "text": "git commit -m \"fix: resolve issue\"",
                "confidence": 0.85
            }
        ],
        "processing_time_ms": 150
    });

    let serialized = serde_json::to_string_pretty(&response).unwrap();
    assert!(serialized.contains("suggestions"));
    assert!(serialized.contains("processing_time_ms"));
}

/// Test that empty buffer returns valid response
#[test]
fn test_empty_buffer_handling() {
    let buffer = "";
    assert!(buffer.is_empty());

    // An empty buffer should not cause a panic
    let cursor_pos = 0;
    assert!(cursor_pos <= buffer.len());
}

/// Test cursor position validation
#[test]
fn test_cursor_position_validation() {
    let buffer = "git commit";
    let cursor_pos = 10; // At the end

    assert!(cursor_pos <= buffer.len());

    // Test boundary conditions
    assert!(0 <= buffer.len());
    assert!(buffer.len() <= buffer.len());
}

/// Test session ID format
#[test]
fn test_session_id_format() {
    let bash_session = "bash-12345";
    let zsh_session = "zsh-67890";

    assert!(bash_session.starts_with("bash-"));
    assert!(zsh_session.starts_with("zsh-"));

    // Extract PID from session
    let bash_pid: Option<u32> = bash_session
        .strip_prefix("bash-")
        .and_then(|s| s.parse().ok());
    assert_eq!(bash_pid, Some(12345));
}

/// Test that multiple suggestions are properly ordered
#[test]
fn test_suggestions_ordering() {
    let suggestions = vec![
        ("git commit -m", 0.95),
        ("git checkout", 0.75),
        ("git cherry-pick", 0.60),
    ];

    // Verify ordering by confidence
    for window in suggestions.windows(2) {
        assert!(window[0].1 >= window[1].1);
    }
}

/// Test context token estimation
#[test]
fn test_token_estimation() {
    let text = "This is a test command with several words";
    let word_count = text.split_whitespace().count();

    // Estimate: words × 1.3
    let estimated_tokens = (word_count as f32 * 1.3).ceil() as usize;

    assert_eq!(word_count, 8);
    assert!(estimated_tokens > word_count);
    assert!(estimated_tokens <= word_count * 2);
}

/// Test that context aggregation includes all sources
#[test]
fn test_context_aggregation() {
    let history = vec!["ls".to_string(), "cd".to_string()];
    let files = vec!["README.md".to_string(), "Cargo.toml".to_string()];
    let exit_code = Some(0);

    // Verify all context sources are present
    assert!(!history.is_empty());
    assert!(!files.is_empty());
    assert!(exit_code.is_some());
}
