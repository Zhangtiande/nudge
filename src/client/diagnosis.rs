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
