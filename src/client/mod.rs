mod diagnosis;
pub mod ipc;

pub use diagnosis::diagnose;

use std::path::PathBuf;

use anyhow::Result;
use tracing::debug;

use crate::cli::OutputFormat;
use crate::protocol::{CompletionRequest, CompletionResponse};

const PLAIN_WARNING_PREFIX: &str = "NUDGE_WARNING:";
const LIST_RISK_HIGH: &str = "high";
const LIST_RISK_LOW: &str = "low";

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
        OutputFormat::List => {
            output_list(&response, &request.buffer);
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

/// Output tab-separated list for popup selectors.
/// Format per line: `<risk>\t<command>\t<warning>\t<why>\t<diff>`
fn output_list(response: &CompletionResponse, buffer: &str) {
    if let Some(text) = build_list_output(response, buffer) {
        print!("{}", text);
    }
}

fn build_list_output(response: &CompletionResponse, buffer: &str) -> Option<String> {
    if response.suggestions.is_empty() {
        return None;
    }

    let mut out = String::new();
    for suggestion in &response.suggestions {
        let risk = if suggestion.warning.is_some() {
            LIST_RISK_HIGH
        } else {
            LIST_RISK_LOW
        };
        let why = build_why(buffer, suggestion);
        let diff = build_diff(buffer, &suggestion.text);
        let warning = suggestion
            .warning
            .as_ref()
            .map(|w| sanitize_list_field(&w.message))
            .unwrap_or_default();
        let command = sanitize_list_field(&suggestion.text);
        out.push_str(risk);
        out.push('\t');
        out.push_str(&command);
        out.push('\t');
        out.push_str(&warning);
        out.push('\t');
        out.push_str(&sanitize_list_field(&why));
        out.push('\t');
        out.push_str(&sanitize_list_field(&diff));
        out.push('\n');
    }

    Some(out)
}

fn sanitize_list_field(input: &str) -> String {
    input
        .replace('\t', " ")
        .replace('\n', " ")
        .replace('\r', " ")
}

fn build_why(buffer: &str, suggestion: &crate::protocol::Suggestion) -> String {
    if suggestion.warning.is_some() {
        return "safety check flagged".to_string();
    }
    if let Some(reason) = &suggestion.reason_short {
        let trimmed = reason.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Some(summary) = &suggestion.summary_short {
        let trimmed = summary.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if suggestion.text.starts_with(buffer) {
        return "prefix completion".to_string();
    }
    "context rewrite".to_string()
}

fn build_diff(buffer: &str, suggestion: &str) -> String {
    if let Some(tail) = suggestion.strip_prefix(buffer) {
        if tail.is_empty() {
            "+<none>".to_string()
        } else {
            format!("+{}", tail)
        }
    } else {
        format!("~ {} -> {}", buffer, suggestion)
    }
}

/// Output full JSON response
fn output_json(response: &CompletionResponse) -> Result<()> {
    let json = serde_json::to_string_pretty(response)?;
    println!("{}", json);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{build_list_output, build_plain_output};
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

    #[test]
    fn test_list_output_emits_risk_command_warning_rows() {
        let response = CompletionResponse::success(
            "req-1".to_string(),
            vec![
                Suggestion::new("git status".to_string()),
                Suggestion::new("rm -rf /".to_string())
                    .with_warning(Warning::dangerous("danger command")),
            ],
            0,
        );

        let output = build_list_output(&response, "git st").unwrap();
        let lines: Vec<&str> = output.lines().collect();
        let cols0: Vec<&str> = lines[0].split('\t').collect();
        let cols1: Vec<&str> = lines[1].split('\t').collect();

        assert_eq!(cols0[0], "low");
        assert_eq!(cols0[1], "git status");
        assert_eq!(cols0[3], "prefix completion");

        assert_eq!(cols1[0], "high");
        assert_eq!(cols1[1], "rm -rf /");
        assert_eq!(cols1[2], "danger command");
        assert_eq!(cols1[3], "safety check flagged");
    }

    #[test]
    fn test_list_output_prefers_summary_for_why_column() {
        let response = CompletionResponse::success(
            "req-2".to_string(),
            vec![Suggestion::new("git status".to_string())
                .with_summary_short("Show working tree status")],
            0,
        );

        let output = build_list_output(&response, "git st").unwrap();
        let cols: Vec<&str> = output.lines().next().unwrap().split('\t').collect();
        assert_eq!(cols[3], "Show working tree status");
    }

    #[test]
    fn test_list_output_prefers_reason_over_summary() {
        let response = CompletionResponse::success(
            "req-3".to_string(),
            vec![Suggestion::new("git status".to_string())
                .with_summary_short("Show working tree status")
                .with_reason_short("matches typed prefix")],
            0,
        );

        let output = build_list_output(&response, "git st").unwrap();
        let cols: Vec<&str> = output.lines().next().unwrap().split('\t').collect();
        assert_eq!(cols[3], "matches typed prefix");
    }
}
