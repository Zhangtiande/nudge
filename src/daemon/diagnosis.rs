use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::config::Config;
use crate::daemon::context::ContextData;

const DIAGNOSIS_SYSTEM_PROMPT: &str = r#"You are a CLI error diagnosis assistant. Analyze the failed command and provide a fix.

Rules:
1. Return ONLY a JSON object with "diagnosis" and "suggestion" fields
2. The "diagnosis" should be a brief (1-2 sentence) explanation starting with an emoji (❌, 💡, ⚠️)
3. The "suggestion" should be the single most likely correct command to fix the error
4. If you cannot determine a fix, set "suggestion" to null
5. Do not explain or add commentary outside the JSON
6. Focus on common issues: typos, missing arguments, wrong paths, permission errors

Example response:
{"diagnosis": "❌ Typo: 'gti' should be 'git'", "suggestion": "git status"}"#;

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct DiagnosisResult {
    diagnosis: String,
    suggestion: Option<String>,
}

/// Diagnose a failed command using LLM
pub async fn diagnose(
    command: &str,
    exit_code: i32,
    stderr: Option<&str>,
    error_record: Option<&serde_json::Value>,
    context: &ContextData,
    config: &Config,
) -> Result<(String, Option<String>)> {
    let client = Client::builder()
        .timeout(Duration::from_millis(config.diagnosis.timeout_ms))
        .build()?;

    let user_prompt = build_diagnosis_prompt(command, exit_code, stderr, error_record, context);

    debug!("Diagnosis prompt: {}", user_prompt);

    let request = ChatRequest {
        model: config.model.model_name.clone(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: DIAGNOSIS_SYSTEM_PROMPT.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_prompt,
            },
        ],
        max_tokens: 200,
        temperature: 0.2,
        stream: false,
    };

    let mut req_builder = client
        .post(format!("{}/chat/completions", config.model.endpoint))
        .json(&request);

    // Add API key
    if let Some(api_key) = &config.model.api_key {
        req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
    } else if let Some(api_key_env) = &config.model.api_key_env {
        if let Ok(api_key) = std::env::var(api_key_env) {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }
    }

    let response = req_builder
        .send()
        .await
        .context("Failed to send diagnosis request")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("LLM request failed with status {}: {}", status, body);
    }

    let completion: ChatResponse = response
        .json()
        .await
        .context("Failed to parse LLM response")?;

    let text = completion
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();

    // Parse JSON response
    parse_diagnosis_response(&text)
}

fn build_diagnosis_prompt(
    command: &str,
    exit_code: i32,
    stderr: Option<&str>,
    error_record: Option<&serde_json::Value>,
    context: &ContextData,
) -> String {
    let mut prompt = String::new();

    prompt.push_str(&format!("## Failed Command\n```\n{}\n```\n\n", command));
    prompt.push_str(&format!("## Exit Code\n{}\n\n", exit_code));

    if let Some(stderr) = stderr {
        if !stderr.is_empty() {
            prompt.push_str("## Error Output (stderr)\n```\n");
            // Truncate if too long
            if stderr.len() > 2000 {
                prompt.push_str(&stderr[..2000]);
                prompt.push_str("\n... (truncated)");
            } else {
                prompt.push_str(stderr);
            }
            prompt.push_str("\n```\n\n");
        }
    }

    if let Some(record) = error_record {
        prompt.push_str("## PowerShell Error Record\n```json\n");
        prompt.push_str(&serde_json::to_string_pretty(record).unwrap_or_default());
        prompt.push_str("\n```\n\n");
    }

    // Add context
    prompt.push_str(&format!(
        "## Current Directory\n{}\n\n",
        context.cwd.display()
    ));

    if !context.files.is_empty() {
        prompt.push_str("## Files in Directory\n");
        prompt.push_str(&context.files.join(", "));
        prompt.push_str("\n\n");
    }

    if !context.history.is_empty() {
        prompt.push_str("## Recent Commands\n");
        for cmd in context.history.iter().take(5) {
            prompt.push_str(&format!("- {}\n", cmd));
        }
        prompt.push('\n');
    }

    prompt.push_str(
        "Analyze the error and respond with JSON only: {\"diagnosis\": \"...\", \"suggestion\": \"...\"}",
    );

    prompt
}

fn parse_diagnosis_response(text: &str) -> Result<(String, Option<String>)> {
    let text = text.trim();

    // Try to extract JSON from response
    let json_str = if text.starts_with('{') {
        text.to_string()
    } else if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            text[start..=end].to_string()
        } else {
            text.to_string()
        }
    } else {
        // No JSON found, return raw text as diagnosis
        return Ok((text.to_string(), None));
    };

    match serde_json::from_str::<DiagnosisResult>(&json_str) {
        Ok(result) => Ok((result.diagnosis, result.suggestion)),
        Err(_) => {
            // Failed to parse, return raw text
            Ok((text.to_string(), None))
        }
    }
}
