use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use interprocess::local_socket::{
    tokio::{prelude::*, Stream},
    ListenerOptions,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::signal;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[cfg(unix)]
use interprocess::local_socket::GenericFilePath;
#[cfg(windows)]
use interprocess::local_socket::GenericNamespaced;

use super::context;
use super::diagnosis;
use super::llm;
use super::safety;
use super::sanitizer;
use super::session::SessionStore;
use super::suggestion_cache::{SuggestionCache, SuggestionKey};
use crate::config::Config;
use crate::protocol::{
    CompletionRequest, CompletionResponse, DiagnosisRequest, DiagnosisResponse, ErrorCode,
    ErrorInfo, Suggestion,
};

/// Wrapper for typed requests
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", content = "payload")]
enum TypedRequest {
    #[serde(rename = "completion")]
    Completion(CompletionRequest),
    #[serde(rename = "diagnosis")]
    Diagnosis(DiagnosisRequest),
}

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
    let cache = Arc::new(Mutex::new(SuggestionCache::new(
        config.cache.capacity,
        config.cache.stale_ratio,
    )));

    // Main accept loop with graceful shutdown
    loop {
        tokio::select! {
            // Accept new connections
            accept_result = listener.accept() => {
                match accept_result {
                    Ok(stream) => {
                        let config = config.clone();
                        let sessions = session_store.clone();
                        let cache = cache.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(stream, config, sessions, cache).await {
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
async fn handle_connection(
    stream: Stream,
    config: Config,
    sessions: SessionStore,
    cache: Arc<Mutex<SuggestionCache>>,
) -> Result<()> {
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

    // Try to parse as typed request first, fall back to completion request
    let typed_request: Result<TypedRequest, _> = serde_json::from_str(&line);

    match typed_request {
        Ok(TypedRequest::Completion(request)) => {
            // Existing completion handling
            debug!(
                "Received completion request from session: {}",
                request.session_id
            );

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

            let response = process_request(request, &config, &sessions, cache.clone()).await;
            let response = CompletionResponse {
                processing_time_ms: start.elapsed().as_millis() as u64,
                ..response
            };
            send_response(&mut writer, &response).await?;
        }
        Ok(TypedRequest::Diagnosis(request)) => {
            // New diagnosis handling
            debug!(
                "Received diagnosis request from session: {}",
                request.session_id
            );
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

            debug!("Received request from session: {}", request.session_id);

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

            let response = process_request(request, &config, &sessions, cache.clone()).await;
            let response = CompletionResponse {
                processing_time_ms: start.elapsed().as_millis() as u64,
                ..response
            };
            send_response(&mut writer, &response).await?;
        }
    }

    debug!("Response sent in {}ms", start.elapsed().as_millis());
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
    cache: Arc<Mutex<SuggestionCache>>,
) -> CompletionResponse {
    let request_id = Uuid::new_v4().to_string();

    // Validate CWD exists
    if !request.cwd.exists() {
        warn!("CWD does not exist: {}", request.cwd.display());
        // Continue anyway - context gathering will handle missing CWD gracefully
    }

    // Update session
    sessions.update_session(&request.session_id, &request.cwd);

    let shell_mode = request
        .shell_mode
        .clone()
        .unwrap_or_else(|| infer_shell_mode(&request.session_id));

    let cache_key = SuggestionKey::build_with_patterns(
        &request,
        request.git_root.as_ref(),
        request.git_state.as_deref(),
        &shell_mode,
        request.time_bucket,
        config.cache.prefix_bytes,
        &config.privacy.custom_patterns,
    );

    let now_ms = now_millis();
    if let Some(hit) = {
        let mut cache = cache.lock().await;
        cache.get_with_state(&cache_key, now_ms)
    } {
        debug!(
            cache_hit = true,
            age_ms = hit.age_ms,
            is_stale = hit.is_stale,
            should_refresh = hit.should_refresh,
            negative = hit.negative,
            "Cache hit"
        );
        let mut response = hit.response;
        response.request_id = request_id;
        response.cache_hit = Some(true);
        response.cache_age_ms = Some(hit.age_ms);

        if hit.should_refresh {
            debug!("Starting background cache refresh (stale-while-revalidate)");
            let refresh_request = request.clone();
            let refresh_config = config.clone();
            let refresh_sessions = sessions.clone();
            let refresh_cache = cache.clone();
            let refresh_key = cache_key.clone();
            let refresh_shell_mode = shell_mode.clone();

            tokio::spawn(async move {
                refresh_sessions.update_session(&refresh_request.session_id, &refresh_request.cwd);
                let response = compute_completion(
                    &refresh_request,
                    &refresh_config,
                    Uuid::new_v4().to_string(),
                )
                .await;
                let insert_now = now_millis();
                let is_negative = response.error.is_some() || response.suggestions.is_empty();
                let ttl_ms = cache_ttl_ms(&refresh_shell_mode, &refresh_config, is_negative);
                debug!(
                    ttl_ms = ttl_ms,
                    "Background refresh complete, updating cache"
                );
                let mut cache = refresh_cache.lock().await;
                cache.insert(refresh_key, response, insert_now, ttl_ms, is_negative);
            });
        }

        return response;
    }

    debug!(cache_hit = false, "Cache miss, computing completion");
    let response = compute_completion(&request, config, request_id.clone()).await;
    let insert_now = now_millis();
    let is_negative = response.error.is_some() || response.suggestions.is_empty();
    let ttl_ms = cache_ttl_ms(&shell_mode, config, is_negative);

    debug!(
        ttl_ms = ttl_ms,
        is_negative = is_negative,
        "Inserting into cache"
    );
    {
        let mut cache = cache.lock().await;
        cache.insert(cache_key, response.clone(), insert_now, ttl_ms, is_negative);
    }

    response
}

async fn compute_completion(
    request: &CompletionRequest,
    config: &Config,
    request_id: String,
) -> CompletionResponse {
    let context_start = Instant::now();

    // Gather context with timing
    let context_result = context::gather(&context::GatherParams::from(request), config).await;
    let context_time = context_start.elapsed();

    if context_time.as_millis() > 50 {
        warn!(
            "Context gathering slow: {}ms (target: <50ms)",
            context_time.as_millis()
        );
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
        debug!("Sanitized {} items", sanitization_event_count);
    }

    // Query LLM
    let llm_start = Instant::now();
    let llm_result = llm::complete(&request.buffer, &sanitized_context, config).await;
    let llm_time = llm_start.elapsed();

    if llm_time.as_millis() > config.model.timeout_ms as u128 / 2 {
        debug!("LLM query: {}ms", llm_time.as_millis());
    }

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

fn cache_ttl_ms(shell_mode: &str, config: &Config, negative: bool) -> u64 {
    if negative {
        return config.cache.ttl_negative_ms;
    }
    if shell_mode.to_lowercase().ends_with("-auto") {
        config.cache.ttl_auto_ms
    } else {
        config.cache.ttl_manual_ms
    }
}

fn infer_shell_mode(session_id: &str) -> String {
    if session_id.starts_with("zsh-") {
        "zsh-inline".to_string()
    } else if session_id.starts_with("bash-") {
        "bash-popup".to_string()
    } else if session_id.starts_with("pwsh-") || session_id.starts_with("powershell-") {
        "ps-inline".to_string()
    } else if session_id.starts_with("cmd-") {
        "cmd-inline".to_string()
    } else {
        "unknown".to_string()
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

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

    // Gather full context for diagnosis (same as completion)
    let context_result = context::gather(&context::GatherParams::from(&request), config).await;
    let context_data = match context_result {
        Ok(ctx) => ctx,
        Err(e) => {
            warn!("Context gathering failed for diagnosis: {}", e);
            // Use empty context
            context::ContextData::default()
        }
    };

    // Sanitize context
    let sanitized_context = if config.privacy.sanitize_enabled {
        let (ctx, _) = sanitizer::sanitize(&context_data, &config.privacy.custom_patterns);
        ctx
    } else {
        context_data
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
        &sanitized_context,
        config,
    )
    .await;

    match diagnosis_result {
        Ok((message, suggestion)) => DiagnosisResponse::success(request_id, message, suggestion, 0),
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

/// Categorize context gathering errors for better user feedback
fn categorize_context_error(error: &anyhow::Error, cwd: &std::path::Path) -> String {
    let error_str = error.to_string().to_lowercase();

    if error_str.contains("permission denied")
        || error_str.contains("no such file")
        || error_str.contains("not found")
    {
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
