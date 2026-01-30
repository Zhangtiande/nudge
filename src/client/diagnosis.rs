use std::path::PathBuf;

use anyhow::Result;

use crate::cli::OutputFormat;
use crate::client::ipc;
use crate::protocol::DiagnosisRequest;

/// Replace emojis with ASCII text for terminals that don't support them well (e.g., Windows PowerShell)
fn sanitize_emojis_for_terminal(text: &str) -> String {
    text.replace("❌", "[Error]")
        .replace("💡", "[Tip]")
        .replace("⚠️", "[Warning]")
        .replace("✅", "[OK]")
}

/// Check if we should sanitize emojis (Windows CMD/PowerShell)
fn should_sanitize_emojis() -> bool {
    #[cfg(windows)]
    {
        // On Windows, check if we're in a terminal that supports Unicode well
        // Windows Terminal and modern consoles support it, but cmd.exe and older PowerShell don't
        // For safety, we sanitize on Windows unless TERM or WT_SESSION indicates modern terminal
        std::env::var("WT_SESSION").is_err() && std::env::var("TERM").is_err()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

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

    // Check if we need to sanitize emojis for this terminal
    let sanitize = should_sanitize_emojis();

    // Output result
    match format {
        OutputFormat::Plain => {
            if let Some(err) = &response.error {
                eprintln!("Diagnosis failed: {}", err.message);
            } else {
                // Print diagnosis message
                if !response.message.is_empty() {
                    let message = if sanitize {
                        sanitize_emojis_for_terminal(&response.message)
                    } else {
                        response.message.clone()
                    };
                    println!("{}", message);
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
