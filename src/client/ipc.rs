use std::time::Duration;

use anyhow::{Context, Result};
use interprocess::local_socket::{
    tokio::{prelude::*, Stream},
    GenericFilePath,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::time::timeout;
use tracing::debug;

use crate::config::Config;
use crate::protocol::{CompletionRequest, CompletionResponse, ErrorCode, ErrorInfo};

/// Connection timeout
const CONNECT_TIMEOUT_MS: u64 = 1000;

/// Read timeout
const READ_TIMEOUT_MS: u64 = 10000;

/// Send completion request to daemon
pub async fn send_request(request: &CompletionRequest) -> Result<CompletionResponse> {
    let socket_path = Config::socket_path();

    // Check if socket exists
    if !socket_path.exists() {
        return Ok(CompletionResponse::error(
            String::new(),
            ErrorInfo::new(
                ErrorCode::LlmUnavailable,
                "Daemon is not running. Start it with: nudge daemon --fork",
                true,
            ),
            0,
        ));
    }

    // Connect with timeout
    let socket_path_str = socket_path.to_string_lossy().to_string();
    let name = socket_path_str.as_str().to_fs_name::<GenericFilePath>()?;

    let connect_result = timeout(
        Duration::from_millis(CONNECT_TIMEOUT_MS),
        Stream::connect(name),
    )
    .await;

    let stream = match connect_result {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            return Ok(CompletionResponse::error(
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
            return Ok(CompletionResponse::error(
                String::new(),
                ErrorInfo::new(
                    ErrorCode::LlmTimeout,
                    "Connection to daemon timed out",
                    true,
                ),
                0,
            ));
        }
    };

    debug!("Connected to daemon");

    // Send request
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);

    let request_json = serde_json::to_string(request)?;
    writer.write_all(request_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    debug!("Request sent, waiting for response");

    // Read response with timeout
    let mut response_line = String::new();
    let read_result = timeout(
        Duration::from_millis(READ_TIMEOUT_MS),
        reader.read_line(&mut response_line),
    )
    .await;

    match read_result {
        Ok(Ok(_)) => {
            let response: CompletionResponse = serde_json::from_str(&response_line)
                .context("Failed to parse daemon response")?;
            debug!("Response received in {}ms", response.processing_time_ms);
            Ok(response)
        }
        Ok(Err(e)) => Ok(CompletionResponse::error(
            String::new(),
            ErrorInfo::new(
                ErrorCode::InternalError,
                format!("Failed to read response: {}", e),
                false,
            ),
            0,
        )),
        Err(_) => Ok(CompletionResponse::error(
            String::new(),
            ErrorInfo::llm_timeout(),
            0,
        )),
    }
}
