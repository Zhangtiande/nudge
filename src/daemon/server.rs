use std::time::Instant;

use anyhow::Result;
use interprocess::local_socket::{
    tokio::{prelude::*, Stream},
    GenericFilePath, ListenerOptions,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::signal;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config::Config;
use crate::protocol::{CompletionRequest, CompletionResponse, ErrorInfo, Suggestion};
use super::context;
use super::llm;
use super::safety;
use super::sanitizer;
use super::session::SessionStore;

/// Run the IPC server
pub async fn run(config: Config) -> Result<()> {
    let socket_path = Config::socket_path();

    // Remove existing socket file if present
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    // Create listener
    let socket_path_str = socket_path.to_string_lossy().to_string();
    let name = socket_path_str.as_str().to_fs_name::<GenericFilePath>()?;
    
    let listener = ListenerOptions::new()
        .name(name)
        .create_tokio()?;

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

    // Cleanup
    let _ = std::fs::remove_file(&socket_path);
    info!("Daemon shutdown complete");

    Ok(())
}

/// Handle a single client connection
async fn handle_connection(
    stream: Stream,
    config: Config,
    sessions: SessionStore,
) -> Result<()> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Read request
    reader.read_line(&mut line).await?;
    let request: CompletionRequest = serde_json::from_str(&line)?;
    debug!("Received request from session: {}", request.session_id);

    let start = Instant::now();
    let request_id = Uuid::new_v4().to_string();

    // Process request
    let response = process_request(request, &config, &sessions).await;

    // Add timing
    let response = CompletionResponse {
        processing_time_ms: start.elapsed().as_millis() as u64,
        ..response
    };

    // Send response
    let response_json = serde_json::to_string(&response)?;
    writer.write_all(response_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    debug!("Response sent in {}ms", response.processing_time_ms);

    Ok(())
}

/// Process a completion request
async fn process_request(
    request: CompletionRequest,
    config: &Config,
    sessions: &SessionStore,
) -> CompletionResponse {
    let request_id = Uuid::new_v4().to_string();

    // Update session
    sessions.update_session(&request.session_id, &request.cwd);

    // Gather context
    let context_result = context::gather(&request, config).await;
    
    let context_data = match context_result {
        Ok(ctx) => ctx,
        Err(e) => {
            warn!("Context gathering failed: {}", e);
            return CompletionResponse::error(
                request_id,
                ErrorInfo::internal_error(format!("Context error: {}", e)),
                0,
            );
        }
    };

    // Sanitize context
    let (sanitized_context, sanitization_events) = if config.privacy.sanitize_enabled {
        sanitizer::sanitize(&context_data, &config.privacy.custom_patterns)
    } else {
        (context_data, vec![])
    };

    // Query LLM
    let llm_result = llm::complete(&request.buffer, &sanitized_context, config).await;

    let suggestion_text = match llm_result {
        Ok(text) => text,
        Err(e) => {
            warn!("LLM completion failed: {}", e);
            return CompletionResponse::error(
                request_id,
                ErrorInfo::llm_unavailable(format!("LLM error: {}", e)),
                0,
            );
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
