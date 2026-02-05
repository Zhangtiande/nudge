use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use super::context::ContextData;
use crate::config::Config;

const DEFAULT_SYSTEM_PROMPT: &str = r#"You are a CLI command completion assistant. Your task is to complete the user's partially typed command based on the provided context.

Rules:
1. Return ONLY the completed command, nothing else
2. Do not explain or add commentary
3. Consider the shell history and current directory context
4. Complete commands that make sense in the given context
5. Prefer safe, non-destructive operations
6. If the command is already complete, return it unchanged

Context will include:
- Recent shell history
- Current working directory files
- Previous command exit status
- Git repository state (if applicable)"#;

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
pub async fn complete(buffer: &str, context: &ContextData, config: &Config) -> Result<String> {
    let client = Client::builder()
        .timeout(Duration::from_millis(config.model.timeout_ms))
        .build()?;

    let system_prompt = config
        .system_prompt
        .as_deref()
        .unwrap_or(DEFAULT_SYSTEM_PROMPT);
    let user_prompt = build_user_prompt(buffer, context);

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

    // Clean up the response - LLM might return with markdown code blocks or extra text
    let cleaned = clean_completion(&text, buffer);

    Ok(cleaned)
}

/// Build the user prompt from context
fn build_user_prompt(buffer: &str, context: &ContextData) -> String {
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
    prompt.push_str("\nComplete the above command. Return ONLY the completed command.");

    prompt
}

/// Clean up LLM response
fn clean_completion(text: &str, original_buffer: &str) -> String {
    let text = text.trim();

    // Remove markdown code blocks if present
    let text = if text.starts_with("```") {
        text.lines()
            .skip(1)
            .take_while(|line| !line.starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        text.to_string()
    };

    // Take only the first line if multiple lines returned
    let text = text.lines().next().unwrap_or(&text).trim();

    // If the completion is empty, return the original buffer
    if text.is_empty() {
        return original_buffer.to_string();
    }

    text.to_string()
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
