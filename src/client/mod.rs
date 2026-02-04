mod diagnosis;
pub mod ipc;

pub use diagnosis::diagnose;

use std::path::PathBuf;

use anyhow::Result;
use tracing::debug;

use crate::cli::OutputFormat;
use crate::protocol::{CompletionRequest, CompletionResponse};

/// Execute completion request
pub async fn complete(
    buffer: String,
    cursor: usize,
    cwd: PathBuf,
    session: String,
    last_exit_code: Option<i32>,
    git_root: Option<PathBuf>,
    git_state: Option<String>,
    shell_mode: Option<String>,
    time_bucket: Option<u64>,
    format: OutputFormat,
) -> Result<()> {
    // Build request
    let mut request = CompletionRequest::new(session, buffer, cursor, cwd, last_exit_code);
    request.git_root = git_root;
    request.git_state = git_state;
    request.shell_mode = shell_mode;
    request.time_bucket = time_bucket;

    debug!("Sending completion request");

    // Send request to daemon
    let response = ipc::send_request(&request).await?;

    // Output based on format
    match format {
        OutputFormat::Plain => {
            output_plain(&response);
        }
        OutputFormat::Json => {
            output_json(&response)?;
        }
    }

    Ok(())
}

/// Output plain text (just the suggestion)
fn output_plain(response: &CompletionResponse) {
    if let Some(suggestion) = response.suggestions.first() {
        // Just print the suggestion text, no newline for shell integration
        print!("{}", suggestion.text);
    }
}

/// Output full JSON response
fn output_json(response: &CompletionResponse) -> Result<()> {
    let json = serde_json::to_string_pretty(response)?;
    println!("{}", json);
    Ok(())
}
