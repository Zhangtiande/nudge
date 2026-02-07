use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, warn};

use super::{context::ContextData, prompts, shell_mode::ShellMode};
use crate::config::Config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionDraft {
    pub command: String,
    pub summary_short: Option<String>,
    pub reason_short: Option<String>,
}

impl CompletionDraft {
    fn from_command(command: String) -> Self {
        Self {
            command,
            summary_short: None,
            reason_short: None,
        }
    }
}

/// LLM API request
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
    stream: bool,
}

/// Chat message
#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// LLM API response
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

/// Response choice
#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

/// Get completion from LLM
pub async fn complete(
    buffer: &str,
    context: &ContextData,
    config: &Config,
    shell_mode: ShellMode,
) -> Result<CompletionDraft> {
    let client = Client::builder()
        .timeout(Duration::from_millis(config.model.timeout_ms))
        .build()?;

    let system_prompt = config
        .system_prompt
        .as_deref()
        .unwrap_or(prompts::completion::default_system_prompt());
    let user_prompt = build_user_prompt(buffer, context, shell_mode);

    // Only log prompts at trace level to avoid flooding logs
    debug!("LLM request: endpoint={}", config.model.endpoint);

    let request = ChatCompletionRequest {
        model: config.model.model_name.clone(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_prompt,
            },
        ],
        max_tokens: 100,
        temperature: 0.3,
        stream: false,
    };

    let mut req_builder = client
        .post(format!("{}/chat/completions", config.model.endpoint))
        .json(&request);

    // Add API key if configured (direct api_key takes precedence over api_key_env)
    if let Some(api_key) = &config.model.api_key {
        req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
    } else if let Some(api_key_env) = &config.model.api_key_env {
        if let Ok(api_key) = std::env::var(api_key_env) {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        } else {
            warn!("API key environment variable {} not set", api_key_env);
        }
    }

    let response = req_builder
        .send()
        .await
        .context("Failed to send request to LLM")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("LLM request failed with status {}: {}", status, body);
    }

    let completion: ChatCompletionResponse = response
        .json()
        .await
        .context("Failed to parse LLM response")?;

    let text = completion
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();

    // Parse completion from model output with backward-compatible plain text fallback.
    let cleaned = parse_completion(&text, buffer);

    Ok(cleaned)
}

/// Build the user prompt from context
fn build_user_prompt(buffer: &str, context: &ContextData, shell_mode: ShellMode) -> String {
    let mut prompt = String::new();

    // Add system information
    prompt.push_str("## System Environment\n");
    prompt.push_str(&format!(
        "OS: {} {}\n",
        context.system.os_type, context.system.os_version
    ));
    prompt.push_str(&format!("Architecture: {}\n", context.system.arch));
    prompt.push_str(&format!("Shell: {}\n", context.system.shell_type));
    prompt.push_str(&format!("User: {}\n\n", context.system.username));

    // Add history
    if !context.history.is_empty() {
        prompt.push_str("## Recent Commands\n");
        for cmd in &context.history {
            prompt.push_str(&format!("- {}\n", cmd));
        }
        prompt.push('\n');
    }

    // Add similar commands (if available)
    if !context.similar_commands.is_empty() {
        prompt.push_str("## Similar Commands from History\n");
        prompt.push_str("The following commands are similar to what you're typing:\n");
        for cmd in &context.similar_commands {
            prompt.push_str(&format!("- {}\n", cmd));
        }
        prompt.push_str("\nConsider these examples, but provide the most appropriate completion based on current context.\n\n");
    }

    // Add CWD listing
    if !context.files.is_empty() {
        prompt.push_str("## Current Directory Files\n");
        let files_str = context.files.join(", ");
        prompt.push_str(&format!("{}\n\n", files_str));
    }

    // Add exit code
    if let Some(exit_code) = context.last_exit_code {
        prompt.push_str(&format!("## Last Command Exit Code: {}\n\n", exit_code));
    }

    // Add git context (legacy)
    if let Some(git) = &context.git {
        prompt.push_str("## Git Status\n");
        if let Some(branch) = &git.branch {
            prompt.push_str(&format!("Branch: {}\n", branch));
        }
        prompt.push_str(&format!("Status: {:?}\n", git.status));
        if !git.staged.is_empty() {
            prompt.push_str(&format!("Staged: {}\n", git.staged.join(", ")));
        }
        prompt.push('\n');
    }

    // Add plugin contexts (new unified approach)
    for (plugin_id, data) in &context.plugins {
        // Format plugin name for display
        let plugin_name = plugin_id
            .chars()
            .next()
            .map(|c| c.to_uppercase().collect::<String>())
            .unwrap_or_default()
            + &plugin_id[1..];

        prompt.push_str(&format!("## {} Context\n", plugin_name));

        // Format plugin data based on type
        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                // Skip internal fields
                if key.starts_with('_') {
                    continue;
                }

                let display_key = humanize_key(key);
                prompt.push_str(&format!("{}: ", display_key));

                match value {
                    serde_json::Value::Bool(b) => {
                        prompt.push_str(&format!("{}\n", if *b { "Yes" } else { "No" }));
                    }
                    serde_json::Value::Number(n) => {
                        prompt.push_str(&format!("{}\n", n));
                    }
                    serde_json::Value::String(s) => {
                        prompt.push_str(&format!("{}\n", s));
                    }
                    serde_json::Value::Array(arr) => {
                        if arr.is_empty() {
                            prompt.push_str("None\n");
                        } else {
                            prompt.push_str(&format_array(arr));
                            prompt.push('\n');
                        }
                    }
                    serde_json::Value::Null => {
                        prompt.push_str("None\n");
                    }
                    _ => {
                        // For complex objects, just indicate presence
                        prompt.push_str("(present)\n");
                    }
                }
            }
        }

        prompt.push('\n');
    }

    // Add the current buffer to complete
    prompt.push_str("## Command to Complete\n");
    prompt.push_str(&format!("```\n{}\n```\n", buffer));
    prompt.push('\n');

    prompt.push_str("## Response Contract\n");
    prompt.push_str(prompts::completion::response_contract(shell_mode));

    prompt
}

/// Parse completion payload from LLM output.
fn parse_completion(text: &str, original_buffer: &str) -> CompletionDraft {
    let text = text.trim();

    // Remove markdown code blocks if present.
    let text = if text.starts_with("```") {
        text.lines()
            .skip(1)
            .take_while(|line| !line.starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        text.to_string()
    };

    if let Some(parsed) = parse_json_completion(&text) {
        return parsed;
    }

    // Take only the first line if multiple lines are returned.
    let text = text.lines().next().unwrap_or(&text).trim();

    // If the completion is empty, return the original buffer.
    if text.is_empty() {
        return CompletionDraft::from_command(original_buffer.to_string());
    }

    CompletionDraft::from_command(text.to_string())
}

fn parse_json_completion(text: &str) -> Option<CompletionDraft> {
    let value: Value = serde_json::from_str(text).ok().or_else(|| {
        text.lines()
            .next()
            .and_then(|line| serde_json::from_str::<Value>(line.trim()).ok())
    })?;
    let obj = value.as_object()?;

    let command = ["command", "text", "completion", "suggestion"]
        .iter()
        .find_map(|key| obj.get(*key).and_then(|v| v.as_str()))
        .map(str::trim)
        .filter(|s| !s.is_empty())?
        .to_string();

    let summary_short = ["summary_short", "summary", "description", "why"]
        .iter()
        .find_map(|key| obj.get(*key).and_then(|v| v.as_str()))
        .and_then(sanitize_summary);
    let reason_short = ["reason_short", "reason", "why"]
        .iter()
        .find_map(|key| obj.get(*key).and_then(|v| v.as_str()))
        .and_then(sanitize_summary);

    Some(CompletionDraft {
        command,
        summary_short,
        reason_short,
    })
}

fn sanitize_summary(summary: &str) -> Option<String> {
    let normalized = summary
        .replace('\t', " ")
        .replace('\n', " ")
        .replace('\r', " ");
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut truncated = trimmed.chars().take(120).collect::<String>();
    if truncated.len() < trimmed.len() {
        truncated.push_str("...");
    }
    Some(truncated)
}

/// Convert snake_case to Title Case
fn humanize_key(key: &str) -> String {
    key.replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format JSON array for display
fn format_array(arr: &[serde_json::Value]) -> String {
    // Limit array output to prevent overwhelming the prompt
    let items: Vec<String> = arr
        .iter()
        .take(10)
        .map(|v| match v {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Object(obj) => {
                // For objects in array, show a concise representation
                let fields: Vec<String> = obj
                    .iter()
                    .take(3)
                    .filter_map(|(k, v)| {
                        if k.starts_with('_') {
                            None
                        } else if let serde_json::Value::String(s) = v {
                            Some(format!("{}={}", k, s))
                        } else {
                            Some(format!("{}={}", k, v))
                        }
                    })
                    .collect();
                if fields.is_empty() {
                    "(object)".to_string()
                } else {
                    format!("[{}]", fields.join(", "))
                }
            }
            _ => v.to_string(),
        })
        .collect();

    if items.is_empty() {
        "None".to_string()
    } else {
        items.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::{build_user_prompt, parse_completion, CompletionDraft};
    use crate::daemon::context::ContextData;
    use crate::daemon::shell_mode::ShellMode;

    #[test]
    fn parse_plain_completion_fallback() {
        let parsed = parse_completion("git status\nextra line", "git st");
        assert_eq!(
            parsed,
            CompletionDraft {
                command: "git status".to_string(),
                summary_short: None,
                reason_short: None
            }
        );
    }

    #[test]
    fn parse_json_completion_with_summary() {
        let parsed = parse_completion(
            r#"{"command":"git status","summary_short":"Check working tree state"}"#,
            "git st",
        );
        assert_eq!(
            parsed,
            CompletionDraft {
                command: "git status".to_string(),
                summary_short: Some("Check working tree state".to_string()),
                reason_short: None
            }
        );
    }

    #[test]
    fn parse_json_completion_with_reason() {
        let parsed = parse_completion(
            r#"{"command":"git status -sb","reason_short":"matches typed git st prefix"}"#,
            "git st",
        );
        assert_eq!(
            parsed,
            CompletionDraft {
                command: "git status -sb".to_string(),
                summary_short: None,
                reason_short: Some("matches typed git st prefix".to_string())
            }
        );
    }

    #[test]
    fn build_prompt_includes_shell_mode_contract() {
        let prompt = build_user_prompt("git st", &ContextData::default(), ShellMode::BashPopup);
        assert!(prompt.contains("Shell mode: bash-popup"));
        assert!(prompt.contains("summary_short"));
    }
}
