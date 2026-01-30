# Error Diagnosis Feature Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement automatic error detection and fix suggestions when shell commands fail.

**Architecture:** Extend the existing IPC protocol with `DiagnosisRequest`/`DiagnosisResponse`, add `nudge diagnose` CLI command, create diagnosis handler in daemon with dedicated LLM prompt, and update shell integrations (Zsh/PowerShell) to capture errors and display suggestions.

**Tech Stack:** Rust (clap, serde, tokio, reqwest), Zsh script, PowerShell script

---

## Task 1: Add Diagnosis Protocol Types

**Files:**
- Modify: `src/protocol.rs`

**Step 1: Add DiagnosisRequest struct**

Add after `CompletionRequest`:

```rust
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
    pub fn new(
        session_id: String,
        command: String,
        exit_code: i32,
        cwd: PathBuf,
    ) -> Self {
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
```

**Step 2: Add DiagnosisResponse struct**

Add after `DiagnosisRequest`:

```rust
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
```

**Step 3: Run format and check**

Run: `cargo fmt && cargo check`
Expected: No errors

**Step 4: Commit**

```bash
git add src/protocol.rs
git commit -m "feat(protocol): add DiagnosisRequest and DiagnosisResponse types"
```

---

## Task 2: Add Diagnosis Configuration

**Files:**
- Modify: `src/config.rs`
- Modify: `config/config.default.yaml.template`

**Step 1: Add DiagnosisConfig struct**

Add after `LogConfig` in `src/config.rs`:

```rust
/// Error diagnosis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DiagnosisConfig {
    /// Enable/disable error diagnosis feature
    pub enabled: bool,
    /// Zsh: capture stderr to file during command execution
    pub capture_stderr: bool,
    /// Zsh: show suggested fix as gray inline text
    pub auto_suggest: bool,
    /// Maximum stderr size to send to LLM (bytes)
    pub max_stderr_size: usize,
    /// Timeout for diagnosis request (ms)
    pub timeout_ms: u64,
}

impl Default for DiagnosisConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            capture_stderr: true,
            auto_suggest: true,
            max_stderr_size: 4096,
            timeout_ms: 5000,
        }
    }
}
```

**Step 2: Add diagnosis field to Config struct**

In `src/config.rs`, add to `Config` struct:

```rust
pub struct Config {
    pub model: ModelConfig,
    pub context: ContextConfig,
    pub plugins: PluginsConfig,
    pub trigger: TriggerConfig,
    pub privacy: PrivacyConfig,
    pub log: LogConfig,
    pub diagnosis: DiagnosisConfig,  // Add this line
    pub system_prompt: Option<String>,
}
```

**Step 3: Update config template**

Add to `config/config.default.yaml.template` before the Advanced section:

```yaml
# ========================================
# Error Diagnosis Configuration
# ========================================
diagnosis:
  # Enable automatic error diagnosis and fix suggestions
  # When enabled, Nudge will analyze failed commands and suggest fixes
  # WARNING (Zsh): stderr is temporarily redirected during command execution
  #   - Error messages won't display in real-time
  #   - After failure, Nudge shows captured errors with diagnosis
  # Default: false (must opt-in)
  enabled: false

  # Zsh: Capture stderr to temporary file for analysis
  # Disable if you experience issues with certain programs
  capture_stderr: true

  # Zsh: Show suggested fix as gray inline text (Tab to accept)
  # Uses the same mechanism as auto-mode suggestions
  auto_suggest: true

  # Maximum stderr size to send to LLM (bytes)
  # Larger errors are truncated to prevent token overflow
  max_stderr_size: 4096

  # Timeout for diagnosis request (milliseconds)
  timeout_ms: 5000
```

**Step 4: Run format and check**

Run: `cargo fmt && cargo check`
Expected: No errors

**Step 5: Commit**

```bash
git add src/config.rs config/config.default.yaml.template
git commit -m "feat(config): add diagnosis configuration options"
```

---

## Task 3: Add Diagnose CLI Subcommand

**Files:**
- Modify: `src/cli.rs`

**Step 1: Add Diagnose command variant**

Add to `Command` enum in `src/cli.rs`:

```rust
    /// Diagnose a failed command and suggest fixes
    Diagnose {
        /// Exit code of the failed command
        #[arg(long)]
        exit_code: i32,

        /// The failed command text
        #[arg(long)]
        command: String,

        /// Current working directory
        #[arg(long)]
        cwd: PathBuf,

        /// Session identifier (e.g., "zsh-12345")
        #[arg(long)]
        session: String,

        /// Path to file containing captured stderr (Zsh)
        #[arg(long)]
        stderr_file: Option<PathBuf>,

        /// PowerShell ErrorRecord as JSON string
        #[arg(long)]
        error_record: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Plain)]
        format: OutputFormat,
    },
```

**Step 2: Run format and check**

Run: `cargo fmt && cargo check`
Expected: No errors

**Step 3: Commit**

```bash
git add src/cli.rs
git commit -m "feat(cli): add diagnose subcommand"
```

---

## Task 4: Implement Diagnosis Client

**Files:**
- Create: `src/client/diagnosis.rs`
- Modify: `src/client/mod.rs`

**Step 1: Create diagnosis client module**

Create `src/client/diagnosis.rs`:

```rust
use std::path::PathBuf;

use anyhow::Result;

use crate::cli::OutputFormat;
use crate::client::ipc;
use crate::protocol::DiagnosisRequest;

/// Send diagnosis request and output result
pub async fn diagnose(
    exit_code: i32,
    command: String,
    cwd: PathBuf,
    session: String,
    stderr_file: Option<PathBuf>,
    error_record: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    // Build request
    let mut request = DiagnosisRequest::new(session, command, exit_code, cwd);

    // Read stderr from file if provided
    if let Some(path) = stderr_file {
        if path.exists() {
            let stderr = std::fs::read_to_string(&path).unwrap_or_default();
            // Truncate if too large (will be configurable later)
            let stderr = if stderr.len() > 4096 {
                stderr[..4096].to_string()
            } else {
                stderr
            };
            request = request.with_stderr(stderr);
        }
    }

    // Parse error record if provided
    if let Some(record_json) = error_record {
        if let Ok(record) = serde_json::from_str(&record_json) {
            request = request.with_error_record(record);
        }
    }

    // Send request
    let response = ipc::send_diagnosis_request(&request).await?;

    // Output result
    match format {
        OutputFormat::Plain => {
            if let Some(err) = &response.error {
                eprintln!("Diagnosis failed: {}", err.message);
            } else {
                // Print diagnosis message
                if !response.message.is_empty() {
                    println!("{}", response.message);
                }
                // Print suggestion on separate line (for shell to capture)
                if let Some(suggestion) = &response.suggestion {
                    println!("{}", suggestion);
                }
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
    }

    Ok(())
}
```

**Step 2: Add send_diagnosis_request to ipc module**

Add to `src/client/ipc.rs`:

```rust
use crate::protocol::{DiagnosisRequest, DiagnosisResponse};

/// Send diagnosis request to daemon
pub async fn send_diagnosis_request(request: &DiagnosisRequest) -> Result<DiagnosisResponse> {
    let socket_path = Config::socket_path();

    // Check daemon alive (same as send_request)
    #[cfg(unix)]
    if !socket_path.exists() {
        return Ok(DiagnosisResponse::error(
            String::new(),
            ErrorInfo::new(
                ErrorCode::LlmUnavailable,
                "Daemon is not running. Start it with: nudge start",
                true,
            ),
            0,
        ));
    }

    if !is_daemon_alive() {
        cleanup_stale_files();
        return Ok(DiagnosisResponse::error(
            String::new(),
            ErrorInfo::new(
                ErrorCode::LlmUnavailable,
                "Daemon is not running. Start it with: nudge start",
                true,
            ),
            0,
        ));
    }

    // Connect
    let socket_path_str = socket_path.to_string_lossy().to_string();

    #[cfg(unix)]
    let name = socket_path_str.as_str().to_fs_name::<GenericFilePath>()?;
    #[cfg(windows)]
    let name = socket_path_str.as_str().to_ns_name::<GenericNamespaced>()?;

    let connect_result = timeout(
        Duration::from_millis(CONNECT_TIMEOUT_MS),
        Stream::connect(name),
    )
    .await;

    let stream = match connect_result {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            return Ok(DiagnosisResponse::error(
                String::new(),
                ErrorInfo::new(
                    ErrorCode::LlmUnavailable,
                    format!("Failed to connect to daemon: {}", e),
                    true,
                ),
                0,
            ));
        }
        Err(_) => {
            return Ok(DiagnosisResponse::error(
                String::new(),
                ErrorInfo::new(ErrorCode::LlmTimeout, "Connection timed out", true),
                0,
            ));
        }
    };

    // Send request with type marker
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);

    // Wrap request with type for daemon to distinguish
    let wrapped = serde_json::json!({
        "type": "diagnosis",
        "payload": request
    });
    let request_json = serde_json::to_string(&wrapped)?;
    writer.write_all(request_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    // Read response
    let mut response_line = String::new();
    let read_result = timeout(
        Duration::from_millis(READ_TIMEOUT_MS),
        reader.read_line(&mut response_line),
    )
    .await;

    match read_result {
        Ok(Ok(_)) => {
            let response: DiagnosisResponse = serde_json::from_str(&response_line)?;
            Ok(response)
        }
        Ok(Err(e)) => Ok(DiagnosisResponse::error(
            String::new(),
            ErrorInfo::new(
                ErrorCode::InternalError,
                format!("Failed to read response: {}", e),
                false,
            ),
            0,
        )),
        Err(_) => Ok(DiagnosisResponse::error(
            String::new(),
            ErrorInfo::llm_timeout(),
            0,
        )),
    }
}
```

**Step 3: Update client mod.rs**

Add to `src/client/mod.rs`:

```rust
mod diagnosis;
pub mod ipc;

pub use diagnosis::diagnose;
// ... existing exports
```

**Step 4: Run format and check**

Run: `cargo fmt && cargo check`
Expected: No errors

**Step 5: Commit**

```bash
git add src/client/diagnosis.rs src/client/mod.rs src/client/ipc.rs
git commit -m "feat(client): implement diagnosis request client"
```

---

## Task 5: Implement Diagnosis LLM Handler

**Files:**
- Create: `src/daemon/diagnosis.rs`
- Modify: `src/daemon/mod.rs`

**Step 1: Create diagnosis LLM module**

Create `src/daemon/diagnosis.rs`:

```rust
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::config::Config;
use crate::daemon::context::ContextData;

const DIAGNOSIS_SYSTEM_PROMPT: &str = r#"You are a CLI error diagnosis assistant. Analyze the failed command and provide a fix.

Rules:
1. Return ONLY a JSON object with "diagnosis" and "suggestion" fields
2. The "diagnosis" should be a brief (1-2 sentence) explanation starting with an emoji (❌, 💡, ⚠️)
3. The "suggestion" should be the single most likely correct command to fix the error
4. If you cannot determine a fix, set "suggestion" to null
5. Do not explain or add commentary outside the JSON
6. Focus on common issues: typos, missing arguments, wrong paths, permission errors

Example response:
{"diagnosis": "❌ Typo: 'gti' should be 'git'", "suggestion": "git status"}"#;

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct DiagnosisResult {
    diagnosis: String,
    suggestion: Option<String>,
}

/// Diagnose a failed command using LLM
pub async fn diagnose(
    command: &str,
    exit_code: i32,
    stderr: Option<&str>,
    error_record: Option<&serde_json::Value>,
    context: &ContextData,
    config: &Config,
) -> Result<(String, Option<String>)> {
    let client = Client::builder()
        .timeout(Duration::from_millis(config.diagnosis.timeout_ms))
        .build()?;

    let user_prompt = build_diagnosis_prompt(command, exit_code, stderr, error_record, context);

    debug!("Diagnosis prompt: {}", user_prompt);

    let request = ChatRequest {
        model: config.model.model_name.clone(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: DIAGNOSIS_SYSTEM_PROMPT.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_prompt,
            },
        ],
        max_tokens: 200,
        temperature: 0.2,
        stream: false,
    };

    let mut req_builder = client
        .post(format!("{}/chat/completions", config.model.endpoint))
        .json(&request);

    // Add API key
    if let Some(api_key) = &config.model.api_key {
        req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
    } else if let Some(api_key_env) = &config.model.api_key_env {
        if let Ok(api_key) = std::env::var(api_key_env) {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }
    }

    let response = req_builder.send().await.context("Failed to send diagnosis request")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("LLM request failed with status {}: {}", status, body);
    }

    let completion: ChatResponse = response.json().await.context("Failed to parse LLM response")?;

    let text = completion
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();

    // Parse JSON response
    parse_diagnosis_response(&text)
}

fn build_diagnosis_prompt(
    command: &str,
    exit_code: i32,
    stderr: Option<&str>,
    error_record: Option<&serde_json::Value>,
    context: &ContextData,
) -> String {
    let mut prompt = String::new();

    prompt.push_str(&format!("## Failed Command\n```\n{}\n```\n\n", command));
    prompt.push_str(&format!("## Exit Code\n{}\n\n", exit_code));

    if let Some(stderr) = stderr {
        if !stderr.is_empty() {
            prompt.push_str("## Error Output (stderr)\n```\n");
            // Truncate if too long
            if stderr.len() > 2000 {
                prompt.push_str(&stderr[..2000]);
                prompt.push_str("\n... (truncated)");
            } else {
                prompt.push_str(stderr);
            }
            prompt.push_str("\n```\n\n");
        }
    }

    if let Some(record) = error_record {
        prompt.push_str("## PowerShell Error Record\n```json\n");
        prompt.push_str(&serde_json::to_string_pretty(record).unwrap_or_default());
        prompt.push_str("\n```\n\n");
    }

    // Add context
    prompt.push_str(&format!("## Current Directory\n{}\n\n", context.cwd.display()));

    if !context.files.is_empty() {
        prompt.push_str("## Files in Directory\n");
        prompt.push_str(&context.files.join(", "));
        prompt.push_str("\n\n");
    }

    if !context.history.is_empty() {
        prompt.push_str("## Recent Commands\n");
        for cmd in context.history.iter().take(5) {
            prompt.push_str(&format!("- {}\n", cmd));
        }
        prompt.push('\n');
    }

    prompt.push_str("Analyze the error and respond with JSON only: {\"diagnosis\": \"...\", \"suggestion\": \"...\"}");

    prompt
}

fn parse_diagnosis_response(text: &str) -> Result<(String, Option<String>)> {
    let text = text.trim();

    // Try to extract JSON from response
    let json_str = if text.starts_with('{') {
        text.to_string()
    } else if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            text[start..=end].to_string()
        } else {
            text.to_string()
        }
    } else {
        // No JSON found, return raw text as diagnosis
        return Ok((text.to_string(), None));
    };

    match serde_json::from_str::<DiagnosisResult>(&json_str) {
        Ok(result) => Ok((result.diagnosis, result.suggestion)),
        Err(_) => {
            // Failed to parse, return raw text
            Ok((text.to_string(), None))
        }
    }
}
```

**Step 2: Update daemon mod.rs**

Add to `src/daemon/mod.rs`:

```rust
pub mod diagnosis;
// ... existing modules
```

**Step 3: Run format and check**

Run: `cargo fmt && cargo check`
Expected: No errors

**Step 4: Commit**

```bash
git add src/daemon/diagnosis.rs src/daemon/mod.rs
git commit -m "feat(daemon): implement diagnosis LLM handler"
```

---

## Task 6: Update Server to Handle Diagnosis Requests

**Files:**
- Modify: `src/daemon/server.rs`

**Step 1: Add request type wrapper**

Add near the top of `src/daemon/server.rs`:

```rust
use crate::protocol::{DiagnosisRequest, DiagnosisResponse};
use super::diagnosis;

/// Wrapper for typed requests
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "payload")]
enum TypedRequest {
    #[serde(rename = "completion")]
    Completion(CompletionRequest),
    #[serde(rename = "diagnosis")]
    Diagnosis(DiagnosisRequest),
}
```

**Step 2: Update handle_connection to dispatch by type**

Replace the request parsing section in `handle_connection`:

```rust
    // Try to parse as typed request first, fall back to completion request
    let typed_request: Result<TypedRequest, _> = serde_json::from_str(&line);

    match typed_request {
        Ok(TypedRequest::Completion(request)) => {
            // Existing completion handling
            let response = process_request(request, &config, &sessions).await;
            let response = CompletionResponse {
                processing_time_ms: start.elapsed().as_millis() as u64,
                ..response
            };
            send_response(&mut writer, &response).await?;
        }
        Ok(TypedRequest::Diagnosis(request)) => {
            // New diagnosis handling
            let response = process_diagnosis_request(request, &config).await;
            let response = DiagnosisResponse {
                processing_time_ms: start.elapsed().as_millis() as u64,
                ..response
            };
            send_diagnosis_response(&mut writer, &response).await?;
        }
        Err(_) => {
            // Fall back to parsing as plain CompletionRequest (backward compatibility)
            let request: CompletionRequest = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    warn!("Invalid request JSON: {}", e);
                    let response = CompletionResponse::error(
                        Uuid::new_v4().to_string(),
                        ErrorInfo::new(
                            ErrorCode::InternalError,
                            format!("{} Error: {}", error_messages::REQUEST_INVALID_JSON, e),
                            false,
                        ),
                        start.elapsed().as_millis() as u64,
                    );
                    send_response(&mut writer, &response).await?;
                    return Ok(());
                }
            };
            let response = process_request(request, &config, &sessions).await;
            let response = CompletionResponse {
                processing_time_ms: start.elapsed().as_millis() as u64,
                ..response
            };
            send_response(&mut writer, &response).await?;
        }
    }
```

**Step 3: Add process_diagnosis_request function**

Add after `process_request`:

```rust
/// Process a diagnosis request
async fn process_diagnosis_request(
    request: DiagnosisRequest,
    config: &Config,
) -> DiagnosisResponse {
    let request_id = Uuid::new_v4().to_string();

    // Check if diagnosis is enabled
    if !config.diagnosis.enabled {
        return DiagnosisResponse::error(
            request_id,
            ErrorInfo::new(
                ErrorCode::ConfigError,
                "Error diagnosis is disabled. Enable with diagnosis.enabled: true",
                false,
            ),
            0,
        );
    }

    // Gather minimal context for diagnosis
    let context_result = context::gather_minimal(&request.cwd, config).await;
    let context_data = match context_result {
        Ok(ctx) => ctx,
        Err(e) => {
            warn!("Context gathering failed for diagnosis: {}", e);
            // Use empty context
            context::ContextData::default()
        }
    };

    // Sanitize stderr if present
    let stderr = request.stderr_output.as_ref().map(|s| {
        if config.privacy.sanitize_enabled {
            let (sanitized, _) = sanitizer::sanitize_string(s, &config.privacy.custom_patterns);
            sanitized
        } else {
            s.clone()
        }
    });

    // Query LLM for diagnosis
    let diagnosis_result = diagnosis::diagnose(
        &request.command,
        request.exit_code,
        stderr.as_deref(),
        request.error_record.as_ref(),
        &context_data,
        config,
    )
    .await;

    match diagnosis_result {
        Ok((message, suggestion)) => {
            DiagnosisResponse::success(request_id, message, suggestion, 0)
        }
        Err(e) => {
            warn!("Diagnosis failed: {}", e);
            DiagnosisResponse::error(
                request_id,
                ErrorInfo::llm_unavailable(format!("Diagnosis failed: {}", e)),
                0,
            )
        }
    }
}

/// Send diagnosis response to client
async fn send_diagnosis_response<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    response: &DiagnosisResponse,
) -> Result<()> {
    let response_json = serde_json::to_string(response)?;
    writer.write_all(response_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}
```

**Step 4: Add gather_minimal to context module**

Add to `src/daemon/context/mod.rs`:

```rust
/// Gather minimal context for diagnosis (faster, less data)
pub async fn gather_minimal(cwd: &Path, config: &Config) -> Result<ContextData> {
    let mut data = ContextData::default();
    data.cwd = cwd.to_path_buf();

    // Only gather CWD listing and recent history
    if let Ok(files) = cwd::list_directory(cwd, config.context.max_files_in_listing) {
        data.files = files;
    }

    if let Ok(history) = history::read_recent(5) {
        data.history = history;
    }

    Ok(data)
}
```

**Step 5: Run format and check**

Run: `cargo fmt && cargo check`
Expected: No errors

**Step 6: Commit**

```bash
git add src/daemon/server.rs src/daemon/context/mod.rs
git commit -m "feat(daemon): handle diagnosis requests in server"
```

---

## Task 7: Wire Up Main Entry Point

**Files:**
- Modify: `src/main.rs`

**Step 1: Add diagnose command handler**

Add to the match block in `main()`:

```rust
        Command::Diagnose {
            exit_code,
            command,
            cwd,
            session,
            stderr_file,
            error_record,
            format,
        } => {
            client::diagnose(exit_code, command, cwd, session, stderr_file, error_record, format).await?;
        }
```

**Step 2: Run format and check**

Run: `cargo fmt && cargo check`
Expected: No errors

**Step 3: Run tests**

Run: `cargo test`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat(main): wire up diagnose command"
```

---

## Task 8: Add Info Command Field for Diagnosis

**Files:**
- Modify: `src/commands/info.rs`

**Step 1: Add diagnosis_enabled field to info output**

Find where info fields are built and add:

```rust
    // Add diagnosis status
    if let Some(ref field) = field {
        if field == "diagnosis_enabled" {
            let config = Config::load().unwrap_or_default();
            println!("{}", config.diagnosis.enabled);
            return Ok(());
        }
    }

    // In JSON output, add:
    // "diagnosis_enabled": config.diagnosis.enabled,
```

**Step 2: Run format and check**

Run: `cargo fmt && cargo check`
Expected: No errors

**Step 3: Commit**

```bash
git add src/commands/info.rs
git commit -m "feat(info): add diagnosis_enabled field"
```

---

## Task 9: Update Zsh Integration

**Files:**
- Modify: `shell/integration.zsh`

**Step 1: Add diagnosis state variables**

Add after existing state variables:

```zsh
# Diagnosis state
typeset -g _nudge_stderr_file=""
typeset -g _nudge_stderr_fd=""
typeset -g _nudge_last_command=""
NUDGE_DIAGNOSIS_ENABLED=$(nudge info --field diagnosis_enabled 2>/dev/null)
```

**Step 2: Add diagnosis preexec hook**

Add new function:

```zsh
# ============================================================================
# Error Diagnosis Functions
# ============================================================================

# Diagnosis preexec - capture stderr before command runs
_nudge_diagnosis_preexec() {
    [[ "$NUDGE_DIAGNOSIS_ENABLED" != "true" ]] && return

    _nudge_last_command="$1"
    _nudge_stderr_file="/tmp/nudge_stderr_$$"

    # Save original stderr and redirect to file
    exec {_nudge_stderr_fd}>&2
    exec 2>"$_nudge_stderr_file"
}

# Diagnosis precmd - analyze errors after command runs
_nudge_diagnosis_precmd() {
    local exit_code=$?

    # Restore stderr immediately
    if [[ -n "$_nudge_stderr_fd" ]]; then
        exec 2>&$_nudge_stderr_fd
        exec {_nudge_stderr_fd}>&-
        _nudge_stderr_fd=""
    fi

    # Only proceed if diagnosis enabled and command failed
    [[ "$NUDGE_DIAGNOSIS_ENABLED" != "true" ]] && return
    [[ $exit_code -eq 0 ]] && return
    [[ -z "$_nudge_last_command" ]] && return

    # Check if stderr file has content
    if [[ -s "$_nudge_stderr_file" ]]; then
        _nudge_ensure_daemon

        # Get diagnosis (format: message\nsuggestion)
        local diagnosis
        diagnosis=$(nudge diagnose \
            --exit-code "$exit_code" \
            --command "$_nudge_last_command" \
            --stderr-file "$_nudge_stderr_file" \
            --cwd "$PWD" \
            --session "zsh-$$" \
            --format plain 2>/dev/null)

        if [[ $? -eq 0 && -n "$diagnosis" ]]; then
            # Split into message and suggestion
            local message="${diagnosis%%$'\n'*}"
            local suggestion="${diagnosis#*$'\n'}"

            # Print diagnosis message (replaces stderr)
            if [[ -n "$message" ]]; then
                echo "$message"
            fi

            # Set suggestion for auto-accept if different from message
            if [[ -n "$suggestion" && "$suggestion" != "$message" ]]; then
                _nudge_auto_suggestion="$suggestion"
            fi
        fi
    fi

    # Cleanup
    rm -f "$_nudge_stderr_file"
    _nudge_stderr_file=""
    _nudge_last_command=""
}
```

**Step 3: Register diagnosis hooks**

Add at the end of the file, before the success message:

```zsh
# Setup diagnosis if enabled
if [[ "$NUDGE_DIAGNOSIS_ENABLED" == "true" ]]; then
    preexec_functions+=(_nudge_diagnosis_preexec)
    # Insert at beginning to capture exit code first
    precmd_functions=(_nudge_diagnosis_precmd "${precmd_functions[@]}")
fi
```

**Step 4: Update success message**

Update the success message section:

```zsh
if [[ -z "$_NUDGE_LOADED" ]]; then
    export _NUDGE_LOADED=1
    local mode_msg=""
    if [[ "$NUDGE_TRIGGER_MODE" == "auto" ]]; then
        mode_msg="auto mode"
    else
        mode_msg="manual mode (Ctrl+E)"
    fi
    if [[ "$NUDGE_DIAGNOSIS_ENABLED" == "true" ]]; then
        mode_msg="$mode_msg + error diagnosis"
    fi
    echo "Nudge loaded ($mode_msg)."
fi
```

**Step 5: Test manually**

```bash
# Source the integration script and test
source shell/integration.zsh
# Run a failing command
gti status  # typo
# Should see diagnosis and suggestion
```

**Step 6: Commit**

```bash
git add shell/integration.zsh
git commit -m "feat(zsh): add error diagnosis integration"
```

---

## Task 10: Update PowerShell Integration

**Files:**
- Modify: `shell/integration.ps1`

**Step 1: Add diagnosis state**

Add after existing state variables:

```powershell
$script:NudgeDiagnosisEnabled = $false
try {
    $diagEnabled = nudge info --field diagnosis_enabled 2>$null
    $script:NudgeDiagnosisEnabled = $diagEnabled -eq "true"
} catch {}

$script:NudgeLastErrorCount = 0
```

**Step 2: Add diagnosis prompt hook**

Add new function:

```powershell
# ============================================================================
# Error Diagnosis Functions
# ============================================================================

function global:Invoke-NudgeDiagnosis {
    if (-not $script:NudgeDiagnosisEnabled) { return }

    $currentErrorCount = $Global:Error.Count

    # Check if new error occurred
    if ($currentErrorCount -gt $script:NudgeLastErrorCount) {
        $lastError = $Global:Error[0]

        if ($lastError) {
            Start-NudgeDaemonIfNeeded

            # Build error record JSON
            $errorContext = @{
                message = $lastError.Exception.Message
                command = $lastError.InvocationInfo.Line
                scriptStackTrace = $lastError.ScriptStackTrace
                category = $lastError.CategoryInfo.ToString()
            } | ConvertTo-Json -Compress

            try {
                $diagnosis = & nudge diagnose `
                    --exit-code $(if ($LASTEXITCODE) { $LASTEXITCODE } else { 1 }) `
                    --command "$($lastError.InvocationInfo.Line)" `
                    --error-record $errorContext `
                    --cwd (Get-Location).Path `
                    --session "pwsh-$PID" `
                    --format plain 2>$null

                if ($LASTEXITCODE -eq 0 -and $diagnosis) {
                    Write-Host $diagnosis -ForegroundColor Yellow
                }
            } catch {
                # Silently ignore diagnosis errors
            }
        }
    }

    $script:NudgeLastErrorCount = $currentErrorCount
}
```

**Step 3: Update prompt hook**

Update the prompt registration section:

```powershell
# Register prompt hook if not already registered
if (-not $global:NudgePromptHookRegistered) {
    $existingPrompt = Get-Content Function:\prompt -ErrorAction SilentlyContinue
    if ($existingPrompt) {
        $newPrompt = @"
Invoke-NudgeCaptureExitCode
Invoke-NudgeDiagnosis
$existingPrompt
"@
        Set-Content Function:\prompt -Value ([scriptblock]::Create($newPrompt))
    }
    $global:NudgePromptHookRegistered = $true
}
```

**Step 4: Update success message**

```powershell
# Print success message
$modeMsg = if ($autoModeEnabled) { "auto mode" } else { "manual mode (Ctrl+E)" }
if ($script:NudgeDiagnosisEnabled) {
    $modeMsg = "$modeMsg + error diagnosis"
}
Write-Host "Nudge loaded ($modeMsg)." -ForegroundColor Green
```

**Step 5: Commit**

```bash
git add shell/integration.ps1
git commit -m "feat(powershell): add error diagnosis integration"
```

---

## Task 11: Update Documentation

**Files:**
- Modify: `README.md`
- Modify: `README_zh.md` (if exists)

**Step 1: Add Error Diagnosis section to README**

Add after the Features section:

```markdown
## Error Diagnosis (v0.5.0+)

Nudge can automatically analyze failed commands and suggest fixes.

### Enable Error Diagnosis

Add to your `config.yaml`:

```yaml
diagnosis:
  enabled: true
```

### How It Works

**Zsh:**
```
$ gti status
❌ Command not found: 'gti'
💡 Typo: did you mean 'git'?

git status          ← gray text, Tab to accept
$ █
```

**PowerShell:**
```
PS> gti status
❌ Command not found: 'gti'
💡 Typo: did you mean 'git'?

PS> █               ← press Ctrl+E for suggestion
```

### ⚠️ Important Notes

> **Zsh Users:** When error diagnosis is enabled, stderr is temporarily redirected
> during command execution. This means:
> - Error messages won't display in real-time
> - After command failure, Nudge displays the captured errors with diagnosis
> - Some programs that check stderr's TTY status may behave differently
>
> If you experience issues, disable with `diagnosis.enabled: false`
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add error diagnosis documentation"
```

---

## Task 12: Update Configuration Templates

**Files:**
- Modify: `config/config.user.yaml.template`

**Step 1: Add diagnosis section to user template**

```yaml
# Error Diagnosis (opt-in)
# diagnosis:
#   enabled: true
```

**Step 2: Commit**

```bash
git add config/config.user.yaml.template
git commit -m "docs: add diagnosis to user config template"
```

---

## Task 13: Final Integration Test

**Step 1: Build release**

Run: `cargo build --release`
Expected: Build succeeds

**Step 2: Run all tests**

Run: `cargo test --verbose`
Expected: All tests pass

**Step 3: Manual end-to-end test**

```bash
# 1. Update config to enable diagnosis
# 2. Restart daemon
nudge restart

# 3. Test in Zsh
source shell/integration.zsh
gti status  # Should show diagnosis

# 4. Test CLI directly
nudge diagnose --exit-code 127 --command "gti status" --cwd . --session test
```

**Step 4: Run pre-commit checks**

Run: `cargo fmt --check && cargo clippy --all-targets -- -D warnings`
Expected: No errors

**Step 5: Final commit**

```bash
git add -A
git commit -m "feat: complete error diagnosis feature implementation"
```

---

## Summary

This plan implements the error diagnosis feature in 13 tasks:

1. **Protocol** - Add `DiagnosisRequest`/`DiagnosisResponse` types
2. **Config** - Add `diagnosis.*` configuration options
3. **CLI** - Add `nudge diagnose` subcommand
4. **Client** - Implement diagnosis request client
5. **LLM Handler** - Create diagnosis-specific LLM module
6. **Server** - Handle diagnosis requests in daemon
7. **Main** - Wire up diagnose command
8. **Info** - Add diagnosis_enabled field
9. **Zsh** - Add stderr capture and diagnosis display
10. **PowerShell** - Add $Error detection and diagnosis
11. **README** - Document the feature with warnings
12. **Templates** - Update config templates
13. **Test** - Full integration test

Estimated implementation time: 2-3 hours for experienced developer.
