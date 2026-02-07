use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Request sent from shell client to daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Unique identifier for the shell session
    pub session_id: String,
    /// ISO 8601 timestamp when request was created
    pub timestamp: DateTime<Utc>,
    /// Current command line buffer content
    pub buffer: String,
    /// Cursor position within buffer (0-indexed)
    pub cursor_pos: usize,
    /// Current working directory absolute path
    pub cwd: PathBuf,
    /// Exit code of the most recent command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_exit_code: Option<i32>,
    /// Git repository root (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_root: Option<PathBuf>,
    /// Git state summary (repo_id|branch|dirty|staged)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_state: Option<String>,
    /// Shell mode (zsh-auto, zsh-inline, ps-inline, bash-popup, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_mode: Option<String>,
    /// Optional time bucket for auto mode (floor(now_ms / 2000))
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_bucket: Option<u64>,
}

impl CompletionRequest {
    pub fn new(
        session_id: String,
        buffer: String,
        cursor_pos: usize,
        cwd: PathBuf,
        last_exit_code: Option<i32>,
    ) -> Self {
        Self {
            session_id,
            timestamp: Utc::now(),
            buffer,
            cursor_pos,
            cwd,
            last_exit_code,
            git_root: None,
            git_state: None,
            shell_mode: None,
            time_bucket: None,
        }
    }
}

/// Response sent from daemon to shell client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Unique identifier for this request (for correlation)
    pub request_id: String,
    /// Array of completion suggestions (ordered by confidence)
    pub suggestions: Vec<Suggestion>,
    /// Total processing time in milliseconds
    pub processing_time_ms: u64,
    /// Error information if completion failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
    /// Summary of context used (optional, for debugging)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_summary: Option<ContextSummary>,
    /// Whether response came from cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_hit: Option<bool>,
    /// Cache age in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_age_ms: Option<u64>,
}

impl CompletionResponse {
    pub fn success(
        request_id: String,
        suggestions: Vec<Suggestion>,
        processing_time_ms: u64,
    ) -> Self {
        Self {
            request_id,
            suggestions,
            processing_time_ms,
            error: None,
            context_summary: None,
            cache_hit: None,
            cache_age_ms: None,
        }
    }

    pub fn error(request_id: String, error: ErrorInfo, processing_time_ms: u64) -> Self {
        Self {
            request_id,
            suggestions: Vec::new(),
            processing_time_ms,
            error: Some(error),
            context_summary: None,
            cache_hit: None,
            cache_age_ms: None,
        }
    }
}

/// A single completion suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// The completed command text
    pub text: String,
    /// Optional short explanation for why this command is suggested
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_short: Option<String>,
    /// Optional concise reason for ranking/selection in selector UIs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_short: Option<String>,
    /// Confidence score (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    /// Warning if command is potentially dangerous
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<Warning>,
}

impl Suggestion {
    pub fn new(text: String) -> Self {
        Self {
            text,
            summary_short: None,
            reason_short: None,
            confidence: None,
            warning: None,
        }
    }

    pub fn with_summary_short(mut self, summary_short: impl Into<String>) -> Self {
        self.summary_short = Some(summary_short.into());
        self
    }

    pub fn with_reason_short(mut self, reason_short: impl Into<String>) -> Self {
        self.reason_short = Some(reason_short.into());
        self
    }

    #[allow(dead_code)]
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence);
        self
    }

    pub fn with_warning(mut self, warning: Warning) -> Self {
        self.warning = Some(warning);
        self
    }
}

/// Warning about a potentially dangerous command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    /// Warning type category
    #[serde(rename = "type")]
    pub warning_type: WarningType,
    /// Human-readable warning message
    pub message: String,
}

impl Warning {
    pub fn dangerous(message: impl Into<String>) -> Self {
        Self {
            warning_type: WarningType::DangerousCommand,
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn irreversible(message: impl Into<String>) -> Self {
        Self {
            warning_type: WarningType::Irreversible,
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn requires_confirmation(message: impl Into<String>) -> Self {
        Self {
            warning_type: WarningType::RequiresConfirmation,
            message: message.into(),
        }
    }
}

/// Warning type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningType {
    DangerousCommand,
    Irreversible,
    RequiresConfirmation,
}

/// Error information when completion fails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error code for programmatic handling
    pub code: ErrorCode,
    /// Human-readable error message
    pub message: String,
    /// Whether the error is recoverable (retry may succeed)
    pub recoverable: bool,
}

impl ErrorInfo {
    pub fn new(code: ErrorCode, message: impl Into<String>, recoverable: bool) -> Self {
        Self {
            code,
            message: message.into(),
            recoverable,
        }
    }

    #[allow(dead_code)]
    pub fn daemon_busy() -> Self {
        Self::new(
            ErrorCode::DaemonBusy,
            "Daemon is busy processing another request",
            true,
        )
    }

    pub fn llm_unavailable(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::LlmUnavailable, msg, true)
    }

    pub fn llm_timeout() -> Self {
        Self::new(ErrorCode::LlmTimeout, "LLM request timed out", true)
    }

    #[allow(dead_code)]
    pub fn config_error(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::ConfigError, msg, false)
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::InternalError, msg, false)
    }
}

/// Error code enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    DaemonBusy,
    LlmUnavailable,
    LlmTimeout,
    ConfigError,
    InternalError,
}

/// Summary of context used for completion (debugging)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextSummary {
    /// Number of history entries used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_count: Option<usize>,
    /// Number of files in CWD listing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_count: Option<usize>,
    /// List of plugin IDs that contributed context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins_used: Option<Vec<String>>,
    /// Estimated total tokens in context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<usize>,
    /// Whether context was truncated due to limits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    /// Number of sensitive items that were redacted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sanitized_count: Option<usize>,
}

/// Context data from plugins (dynamic JSON structure)
#[allow(dead_code)]
pub type PluginContext = HashMap<String, serde_json::Value>;

/// Request for error diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisRequest {
    /// Unique identifier for the shell session
    pub session_id: String,
    /// ISO 8601 timestamp when request was created
    pub timestamp: DateTime<Utc>,
    /// The failed command text
    pub command: String,
    /// Exit code of the failed command
    pub exit_code: i32,
    /// Captured stderr output (Zsh)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_output: Option<String>,
    /// PowerShell ErrorRecord as JSON
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_record: Option<serde_json::Value>,
    /// Current working directory absolute path
    pub cwd: PathBuf,
}

impl DiagnosisRequest {
    pub fn new(session_id: String, command: String, exit_code: i32, cwd: PathBuf) -> Self {
        Self {
            session_id,
            timestamp: Utc::now(),
            command,
            exit_code,
            stderr_output: None,
            error_record: None,
            cwd,
        }
    }

    pub fn with_stderr(mut self, stderr: String) -> Self {
        self.stderr_output = Some(stderr);
        self
    }

    pub fn with_error_record(mut self, record: serde_json::Value) -> Self {
        self.error_record = Some(record);
        self
    }
}

/// Response with diagnosis and suggested fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisResponse {
    /// Unique identifier for this request
    pub request_id: String,
    /// Human-readable diagnosis message
    pub message: String,
    /// Suggested fix command (single best option)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    /// Confidence score (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Error if diagnosis failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

impl DiagnosisResponse {
    pub fn success(
        request_id: String,
        message: String,
        suggestion: Option<String>,
        processing_time_ms: u64,
    ) -> Self {
        Self {
            request_id,
            message,
            suggestion,
            confidence: None,
            processing_time_ms,
            error: None,
        }
    }

    pub fn error(request_id: String, error: ErrorInfo, processing_time_ms: u64) -> Self {
        Self {
            request_id,
            message: String::new(),
            suggestion: None,
            confidence: None,
            processing_time_ms,
            error: Some(error),
        }
    }
}
