use std::time::Instant;

use anyhow::Result;
use interprocess::local_socket::{
    tokio::{prelude::*, Stream},
    ListenerOptions,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::signal;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[cfg(unix)]
use interprocess::local_socket::GenericFilePath;
#[cfg(windows)]
use interprocess::local_socket::GenericNamespaced;

use super::context;
use super::llm;
use super::safety;
use super::sanitizer;
use super::session::SessionStore;
use crate::config::Config;
use crate::protocol::{CompletionRequest, CompletionResponse, ErrorCode, ErrorInfo, Suggestion};

/// Common error messages for better user experience
#[allow(dead_code)]
mod error_messages {
    pub const SOCKET_PERMISSION_DENIED: &str =
        "Permission denied when creating socket. Check that the directory exists and is writable: ";
    pub const SOCKET_ALREADY_IN_USE: &str =
        "Socket is already in use. Another daemon instance may be running. Try 'nudge stop' first.";
    pub const CONFIG_NOT_FOUND: &str =
        "Configuration file not found. Using default settings. Create config at: ";
    pub const CONFIG_PARSE_ERROR: &str =
        "Failed to parse configuration file. Check YAML syntax at: ";
    pub const LLM_CONNECTION_REFUSED: &str =
        "Cannot connect to LLM endpoint. Ensure the LLM server (e.g., Ollama) is running at: ";
    pub const LLM_TIMEOUT: &str =
        "LLM request timed out. The model may be overloaded or the timeout is too short.";
    pub const LLM_AUTH_FAILED: &str =
        "LLM authentication failed. Check your API key environment variable: ";
    pub const CONTEXT_CWD_NOT_FOUND: &str = "Current working directory not accessible: ";
    pub const CONTEXT_HISTORY_UNREADABLE: &str =
        "Cannot read shell history file. Completion will work without history context.";
    pub const GIT_TIMEOUT: &str = "Git operations timed out (>50ms). Git context will be excluded.";
    pub const REQUEST_INVALID_JSON: &str =
        "Invalid request format. Expected JSON with session_id, buffer, cursor_pos, cwd fields.";
    pub const REQUEST_BUFFER_TOO_LARGE: &str =
        "Command buffer exceeds maximum size (10000 characters).";
}

/// Run the IPC server
pub async fn run(config: Config) -> Result<()> {
    let socket_path = Config::socket_path();

    // Remove existing socket file if present (Unix only, Windows Named Pipes don't leave files)
    #[cfg(unix)]
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    // Create listener
    let socket_path_str = socket_path.to_string_lossy().to_string();

    #[cfg(unix)]
    let name = socket_path_str.as_str().to_fs_name::<GenericFilePath>()?;
    #[cfg(windows)]
    let name = socket_path_str.as_str().to_ns_name::<GenericNamespaced>()?;

    let listener = ListenerOptions::new().name(name).create_tokio()?;

    info!("Listening on {}", socket_path.display());

    // Create shared state
    let session_store = SessionStore::new();

    // Main accept loop with graceful shutdown
    loop {
        tokio::select! {
            // Accept new connections
            accept_result = listener.accept() => {
                match accept_result {
                    Ok(stream) => {
                        let config = config.clone();
                        let sessions = session_store.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(stream, config, sessions).await {
                                error!("Connection handler error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Accept error: {}", e);
                    }
                }
            }

            // Handle shutdown signals
            _ = signal::ctrl_c() => {
                info!("Received shutdown signal");
                break;
            }
        }
    }

    // Cleanup (Unix only, Windows Named Pipes don't leave files)
    #[cfg(unix)]
    let _ = std::fs::remove_file(&socket_path);
    info!("Daemon shutdown complete");

    Ok(())
}

/// Handle a single client connection
async fn handle_connection(stream: Stream, config: Config, sessions: SessionStore) -> Result<()> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    let start = Instant::now();

    // Read request with improved error handling
    if let Err(e) = reader.read_line(&mut line).await {
        error!("Failed to read request: {}", e);
        let response = CompletionResponse::error(
            Uuid::new_v4().to_string(),
            ErrorInfo::new(
                ErrorCode::InternalError,
                format!("Read error: {}", e),
                false,
            ),
            start.elapsed().as_millis() as u64,
        );
        send_response(&mut writer, &response).await?;
        return Ok(());
    }

    // Parse request with helpful error message
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

    // Validate buffer size
    if request.buffer.len() > 10000 {
        warn!("Buffer too large: {} bytes", request.buffer.len());
        let response = CompletionResponse::error(
            Uuid::new_v4().to_string(),
            ErrorInfo::new(
                ErrorCode::InternalError,
                error_messages::REQUEST_BUFFER_TOO_LARGE,
                false,
            ),
            start.elapsed().as_millis() as u64,
        );
        send_response(&mut writer, &response).await?;
        return Ok(());
    }

    debug!("Received request from session: {}", request.session_id);

    // Process request
    let response = process_request(request, &config, &sessions).await;

    // Add timing
    let response = CompletionResponse {
        processing_time_ms: start.elapsed().as_millis() as u64,
        ..response
    };

    send_response(&mut writer, &response).await?;
    debug!("Response sent in {}ms", response.processing_time_ms);

    Ok(())
}

/// Send response to client
async fn send_response<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    response: &CompletionResponse,
) -> Result<()> {
    let response_json = serde_json::to_string(response)?;
    writer.write_all(response_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

/// Process a completion request
async fn process_request(
    request: CompletionRequest,
    config: &Config,
    sessions: &SessionStore,
) -> CompletionResponse {
    let request_id = Uuid::new_v4().to_string();
    let context_start = Instant::now();

    // Validate CWD exists
    if !request.cwd.exists() {
        warn!("CWD does not exist: {}", request.cwd.display());
        // Continue anyway - context gathering will handle missing CWD gracefully
    }

    // Update session
    sessions.update_session(&request.session_id, &request.cwd);

    // Gather context with timing
    let context_result = context::gather(&request, config).await;
    let context_time = context_start.elapsed();

    if context_time.as_millis() > 50 {
        warn!(
            "Context gathering took {}ms (target: <50ms)",
            context_time.as_millis()
        );
    } else {
        debug!("Context gathered in {}ms", context_time.as_millis());
    }

    let context_data = match context_result {
        Ok(ctx) => ctx,
        Err(e) => {
            let error_msg = categorize_context_error(&e, &request.cwd);
            warn!("Context gathering failed: {} ({})", error_msg, e);
            return CompletionResponse::error(request_id, ErrorInfo::internal_error(error_msg), 0);
        }
    };

    // Sanitize context
    let (sanitized_context, sanitization_event_count) = if config.privacy.sanitize_enabled {
        let (ctx, events) = sanitizer::sanitize(&context_data, &config.privacy.custom_patterns);
        (ctx, events.len())
    } else {
        (context_data, 0)
    };

    if sanitization_event_count > 0 {
        debug!("Sanitized {} sensitive items", sanitization_event_count);
    }

    // Query LLM with improved error categorization
    let llm_start = Instant::now();
    let llm_result = llm::complete(&request.buffer, &sanitized_context, config).await;
    let llm_time = llm_start.elapsed();

    debug!("LLM query completed in {}ms", llm_time.as_millis());

    let suggestion_text = match llm_result {
        Ok(text) => text,
        Err(e) => {
            let (error_info, log_msg) = categorize_llm_error(&e, config);
            warn!("LLM completion failed: {}", log_msg);
            return CompletionResponse::error(request_id, error_info, 0);
        }
    };

    // Check for dangerous commands
    let warning = if config.privacy.block_dangerous {
        safety::check(&suggestion_text, &config.privacy.custom_blocked)
    } else {
        None
    };

    // Build response
    let mut suggestion = Suggestion::new(suggestion_text);
    if let Some(w) = warning {
        suggestion = suggestion.with_warning(w);
    }

    CompletionResponse::success(request_id, vec![suggestion], 0)
}

/// Categorize context gathering errors for better user feedback
fn categorize_context_error(error: &anyhow::Error, cwd: &std::path::Path) -> String {
    let error_str = error.to_string().to_lowercase();

    if error_str.contains("permission denied") {
        format!("{}{}", error_messages::CONTEXT_CWD_NOT_FOUND, cwd.display())
    } else if error_str.contains("no such file") || error_str.contains("not found") {
        format!("{}{}", error_messages::CONTEXT_CWD_NOT_FOUND, cwd.display())
    } else if error_str.contains("history") {
        error_messages::CONTEXT_HISTORY_UNREADABLE.to_string()
    } else if error_str.contains("timeout") || error_str.contains("timed out") {
        error_messages::GIT_TIMEOUT.to_string()
    } else {
        format!("Context error: {}", error)
    }
}

/// Categorize LLM errors for better user feedback
fn categorize_llm_error(error: &anyhow::Error, config: &Config) -> (ErrorInfo, String) {
    let error_str = error.to_string().to_lowercase();

    if error_str.contains("connection refused") || error_str.contains("connect error") {
        let msg = format!(
            "{}{}",
            error_messages::LLM_CONNECTION_REFUSED,
            config.model.endpoint
        );
        (ErrorInfo::llm_unavailable(&msg), msg)
    } else if error_str.contains("timeout") || error_str.contains("timed out") {
        let msg = format!(
            "{} Current timeout: {}ms. Consider increasing model.timeout_ms in config.",
            error_messages::LLM_TIMEOUT,
            config.model.timeout_ms
        );
        (ErrorInfo::llm_timeout(), msg)
    } else if error_str.contains("401")
        || error_str.contains("unauthorized")
        || error_str.contains("authentication")
    {
        let api_key_env = config
            .model
            .api_key_env
            .as_deref()
            .unwrap_or("(not configured)");
        let msg = format!("{}{}", error_messages::LLM_AUTH_FAILED, api_key_env);
        (ErrorInfo::llm_unavailable(&msg), msg)
    } else if error_str.contains("404") || error_str.contains("not found") {
        let msg = format!(
            "Model '{}' not found at endpoint '{}'. Check model name and endpoint configuration.",
            config.model.model_name, config.model.endpoint
        );
        (ErrorInfo::llm_unavailable(&msg), msg)
    } else if error_str.contains("429") || error_str.contains("rate limit") {
        let msg = "Rate limit exceeded. Try again later or use a local model.".to_string();
        (ErrorInfo::new(ErrorCode::LlmUnavailable, &msg, true), msg)
    } else {
        let msg = format!("LLM error: {}", error);
        (ErrorInfo::llm_unavailable(&msg), msg)
    }
}
