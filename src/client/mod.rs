mod diagnosis;
pub mod ipc;

pub use diagnosis::diagnose;

use std::path::PathBuf;

use anyhow::Result;
use tracing::debug;

use crate::cli::OutputFormat;
use crate::protocol::{CompletionRequest, CompletionResponse};

const PLAIN_WARNING_PREFIX: &str = "NUDGE_WARNING:";

/// Execute completion request
#[allow(clippy::too_many_arguments)]
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
    if let Some(text) = build_plain_output(response) {
        // Just print the suggestion text, no newline for shell integration
        print!("{}", text);
    }
}

fn build_plain_output(response: &CompletionResponse) -> Option<String> {
    response.suggestions.first().map(|suggestion| {
        if let Some(warning) = &suggestion.warning {
            format!("{} {}", PLAIN_WARNING_PREFIX, warning.message)
        } else {
            suggestion.text.clone()
        }
    })
}

/// Output full JSON response
fn output_json(response: &CompletionResponse) -> Result<()> {
    let json = serde_json::to_string_pretty(response)?;
    println!("{}", json);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::build_plain_output;
    use crate::protocol::{CompletionResponse, Suggestion, Warning};

    #[test]
    fn test_plain_output_emits_warning_sentinel() {
        let response = CompletionResponse::success(
            "req-1".to_string(),
            vec![Suggestion::new("rm -rf /".to_string()).with_warning(Warning::dangerous("danger"))],
            0,
        );

        let output = build_plain_output(&response);

        assert_eq!(output, Some("NUDGE_WARNING: danger".to_string()));
    }

    #[test]
    fn test_plain_output_uses_suggestion_text_when_safe() {
        let response = CompletionResponse::success(
            "req-1".to_string(),
            vec![Suggestion::new("git status".to_string())],
            0,
        );

        let output = build_plain_output(&response);

        assert_eq!(output, Some("git status".to_string()));
    }
}
