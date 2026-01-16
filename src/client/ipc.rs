use std::time::Duration;

use anyhow::{Context, Result};
use interprocess::local_socket::tokio::{prelude::*, Stream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::time::timeout;
use tracing::debug;

#[cfg(unix)]
use interprocess::local_socket::GenericFilePath;
#[cfg(windows)]
use interprocess::local_socket::GenericNamespaced;

use crate::config::Config;
use crate::protocol::{CompletionRequest, CompletionResponse, ErrorCode, ErrorInfo};

/// Connection timeout
const CONNECT_TIMEOUT_MS: u64 = 1000;

/// Read timeout
const READ_TIMEOUT_MS: u64 = 10000;

/// Check if daemon process is actually running (not just socket file exists)
fn is_daemon_alive() -> bool {
    let pid_path = Config::pid_path();

    if !pid_path.exists() {
        return false;
    }

    if let Ok(pid_str) = std::fs::read_to_string(&pid_path) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            return is_process_alive(pid as u32);
        }
    }

    false
}

/// Check if a process with given PID is alive (Unix implementation)
#[cfg(unix)]
fn is_process_alive(pid: u32) -> bool {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;
    // Signal 0 checks if process exists without sending actual signal
    kill(Pid::from_raw(pid as i32), Signal::SIGCONT).is_ok()
}

/// Check if a process with given PID is alive (Windows implementation)
#[cfg(windows)]
fn is_process_alive(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if !handle.is_null() {
            CloseHandle(handle);
            true
        } else {
            false
        }
    }
}

/// Clean up stale socket and pid files
fn cleanup_stale_files() {
    // Unix: Remove socket file (Named Pipes on Windows don't leave files)
    #[cfg(unix)]
    let _ = std::fs::remove_file(Config::socket_path());

    // Always clean up PID file
    let _ = std::fs::remove_file(Config::pid_path());

    debug!("Cleaned up stale socket/pid files");
}

/// Send completion request to daemon
pub async fn send_request(request: &CompletionRequest) -> Result<CompletionResponse> {
    let socket_path = Config::socket_path();

    // Unix: Check if socket file exists (doesn't work for Windows Named Pipes)
    #[cfg(unix)]
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

    // Check if daemon process is actually alive before attempting connection
    // This prevents blocking on a stale socket file (Unix) or invalid pipe (Windows)
    if !is_daemon_alive() {
        cleanup_stale_files();
        return Ok(CompletionResponse::error(
            String::new(),
            ErrorInfo::new(
                ErrorCode::LlmUnavailable,
                "Daemon is not running (stale socket cleaned). Start it with: nudge daemon --fork",
                true,
            ),
            0,
        ));
    }

    // Connect with timeout
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
            let response: CompletionResponse =
                serde_json::from_str(&response_line).context("Failed to parse daemon response")?;
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
