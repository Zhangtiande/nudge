use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;

use crate::config::Config;
use crate::daemon::{context, sanitizer};
use crate::protocol::CompletionRequest;

#[derive(Debug, Serialize)]
struct ContextOutput {
    buffer: String,
    session: String,
    cwd: PathBuf,
    last_exit_code: Option<i32>,
    sanitize_enabled: bool,
    sanitized_count: usize,
    context: context::ContextData,
}

pub async fn run_context(
    buffer: String,
    cwd: PathBuf,
    session: String,
    last_exit_code: Option<i32>,
    json: bool,
) -> Result<()> {
    let config = Config::load().unwrap_or_default();

    // Build the same request shape as completion, then reuse GatherParams::from(&CompletionRequest)
    // to ensure context collection behavior stays aligned with completion.
    let request = CompletionRequest::new(
        session.clone(),
        buffer.clone(),
        buffer.len(),
        cwd.clone(),
        last_exit_code,
    );
    let params = context::GatherParams::from(&request);
    let gathered_context = context::gather(&params, &config).await?;

    let (effective_context, sanitized_count) = if config.privacy.sanitize_enabled {
        let (sanitized_context, events) =
            sanitizer::sanitize(&gathered_context, &config.privacy.custom_patterns);
        (sanitized_context, events.len())
    } else {
        (gathered_context, 0)
    };

    let output = ContextOutput {
        buffer,
        session,
        cwd,
        last_exit_code,
        sanitize_enabled: config.privacy.sanitize_enabled,
        sanitized_count,
        context: effective_context,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Nudge Completion Context");
        println!("========================");
        println!("buffer: {}", output.buffer);
        println!("session: {}", output.session);
        println!("cwd: {}", output.cwd.display());
        println!("last_exit_code: {:?}", output.last_exit_code);
        println!("sanitize_enabled: {}", output.sanitize_enabled);
        println!("sanitized_count: {}", output.sanitized_count);
        println!();
        println!("{}", serde_json::to_string_pretty(&output.context)?);
    }

    Ok(())
}
