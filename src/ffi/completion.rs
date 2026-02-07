//! FFI completion logic
//!
//! This module implements the completion logic for FFI calls, reusing
//! the existing daemon code for context gathering, sanitization, and LLM calls.

use std::path::PathBuf;

use crate::config::Config;
use crate::daemon::context::{self, GatherParams};
use crate::daemon::llm;
use crate::daemon::safety;
use crate::daemon::sanitizer;
use crate::daemon::shell_mode::ShellMode;
use crate::protocol::CompletionRequest;

/// Result of a completion operation
pub struct CompletionResult {
    /// The completed command suggestion
    pub suggestion: String,
    /// Warning message if the command is dangerous
    pub warning: Option<String>,
    /// Error message if completion failed
    pub error: Option<String>,
}

impl CompletionResult {
    /// Create a successful result
    pub fn success(suggestion: String, warning: Option<String>) -> Self {
        Self {
            suggestion,
            warning,
            error: None,
        }
    }

    /// Create an error result
    pub fn error(message: String) -> Self {
        Self {
            suggestion: String::new(),
            warning: None,
            error: Some(message),
        }
    }
}

/// Perform completion using the daemon's logic
///
/// This function:
/// 1. Gathers context (history, CWD, git, plugins)
/// 2. Sanitizes sensitive data
/// 3. Calls the LLM API
/// 4. Checks for dangerous commands
///
/// # Arguments
/// * `buffer` - Current command line buffer
/// * `cursor` - Cursor position in buffer
/// * `cwd` - Current working directory
/// * `session_id` - Shell session identifier
/// * `config` - Loaded configuration
pub async fn complete(
    buffer: &str,
    cursor: usize,
    cwd: &str,
    session_id: &str,
    config: &Config,
) -> CompletionResult {
    // Create completion request
    let request = CompletionRequest::new(
        session_id.to_string(),
        buffer.to_string(),
        cursor,
        PathBuf::from(cwd),
        None, // last_exit_code not available in FFI mode
    );

    // Gather context
    let params = GatherParams::from(&request);
    let context = match context::gather(&params, config).await {
        Ok(ctx) => ctx,
        Err(e) => {
            return CompletionResult::error(format!("Failed to gather context: {}", e));
        }
    };

    // Sanitize context
    let (sanitized_context, _events) = if config.privacy.sanitize_enabled {
        sanitizer::sanitize(&context, &config.privacy.custom_patterns)
    } else {
        (context, Vec::new())
    };

    // Call LLM
    let shell_mode = ShellMode::resolve(None, session_id);
    let completion = match llm::complete(buffer, &sanitized_context, config, shell_mode).await {
        Ok(s) => s,
        Err(e) => {
            return CompletionResult::error(format!("LLM completion failed: {}", e));
        }
    };
    let suggestion = completion.command;

    // Check for dangerous commands
    let warning = if config.privacy.block_dangerous {
        safety::check(&suggestion, &config.privacy.custom_blocked).map(|w| w.message)
    } else {
        None
    };

    CompletionResult::success(suggestion, warning)
}

/// Simple hash function for cache keys
pub fn hash_input(buffer: &str, cwd: &str, session_id: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    buffer.hash(&mut hasher);
    cwd.hash(&mut hasher);
    session_id.hash(&mut hasher);
    hasher.finish()
}
