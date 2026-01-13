pub mod history;
pub mod cwd;
pub mod plugin;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::config::Config;
use crate::protocol::CompletionRequest;
use super::plugins::git::GitContext;

/// Aggregated context data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextData {
    /// Recent command history
    pub history: Vec<String>,
    /// Files in current directory
    pub files: Vec<String>,
    /// Exit code of last command
    pub last_exit_code: Option<i32>,
    /// Git context (if applicable)
    pub git: Option<GitContext>,
    /// Estimated token count
    pub estimated_tokens: usize,
}

impl ContextData {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Gather all context for a completion request
pub async fn gather(request: &CompletionRequest, config: &Config) -> Result<ContextData> {
    let mut context = ContextData::new();

    // Gather history
    let history = history::read_history(&request.session_id, config.context.history_window)?;
    context.history = history;
    debug!("Gathered {} history entries", context.history.len());

    // Gather CWD listing
    if config.context.include_cwd_listing {
        let files = cwd::list_files(&request.cwd, config.context.max_files_in_listing)?;
        context.files = files;
        debug!("Gathered {} files from CWD", context.files.len());
    }

    // Set exit code
    if config.context.include_exit_code {
        context.last_exit_code = request.last_exit_code;
    }

    // Gather git context
    if config.plugins.git.enabled {
        match super::plugins::git::collect(&request.cwd, &config.plugins.git).await {
            Ok(git_ctx) => {
                context.git = Some(git_ctx);
                debug!("Gathered git context");
            }
            Err(e) => {
                debug!("Git context not available: {}", e);
            }
        }
    }

    // Estimate tokens
    context.estimated_tokens = estimate_tokens(&context);

    // Truncate if necessary
    if context.estimated_tokens > config.context.max_total_tokens {
        truncate_by_priority(&mut context, config);
    }

    Ok(context)
}

/// Estimate token count (word-based approximation)
fn estimate_tokens(context: &ContextData) -> usize {
    let mut total = 0;

    // History: count words × 1.3
    for cmd in &context.history {
        total += (cmd.split_whitespace().count() as f32 * 1.3).ceil() as usize;
    }

    // Files: roughly 1 token per file name
    total += context.files.len();

    // Git context: estimate based on content
    if let Some(git) = &context.git {
        if git.branch.is_some() {
            total += 5;
        }
        total += git.staged.len();
        total += git.recent_commits.len() * 10;
    }

    total
}

/// Truncate context by priority
fn truncate_by_priority(context: &mut ContextData, config: &Config) {
    let priorities = &config.context.priorities;
    let max_tokens = config.context.max_total_tokens;

    // Sort sources by priority (lowest first for removal)
    let mut sources: Vec<(&str, u8)> = vec![
        ("history", priorities.history),
        ("cwd_listing", priorities.cwd_listing),
        ("plugins", priorities.plugins),
    ];
    sources.sort_by_key(|(_, p)| *p);

    for (source, _) in sources {
        if context.estimated_tokens <= max_tokens {
            break;
        }

        match source {
            "plugins" => {
                // Remove git context first (lowest priority)
                if context.git.is_some() {
                    context.git = None;
                    context.estimated_tokens = estimate_tokens(context);
                }
            }
            "cwd_listing" => {
                // Remove half of files
                let half = context.files.len() / 2;
                context.files.truncate(half);
                context.estimated_tokens = estimate_tokens(context);
            }
            "history" => {
                // Remove oldest history entries
                let half = context.history.len() / 2;
                if half > 0 {
                    context.history = context.history.split_off(context.history.len() - half);
                    context.estimated_tokens = estimate_tokens(context);
                }
            }
            _ => {}
        }
    }
}
