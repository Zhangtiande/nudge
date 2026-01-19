pub mod cwd;
pub mod history;
pub mod plugin;
pub mod system;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::plugins::git::GitContext;
use crate::config::Config;
use crate::protocol::CompletionRequest;
use system::SystemInfo;

/// Aggregated context data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextData {
    /// Recent command history
    pub history: Vec<String>,
    /// Similar commands from history
    pub similar_commands: Vec<String>,
    /// Files in current directory
    pub files: Vec<String>,
    /// Exit code of last command
    pub last_exit_code: Option<i32>,
    /// Git context (if applicable)
    pub git: Option<GitContext>,
    /// System information
    pub system: SystemInfo,
    /// Estimated token count
    pub estimated_tokens: usize,
}

impl Default for ContextData {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            similar_commands: Vec::new(),
            files: Vec::new(),
            last_exit_code: None,
            git: None,
            system: SystemInfo::default(),
            estimated_tokens: 0,
        }
    }
}

impl ContextData {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Gather all context for a completion request
pub async fn gather(request: &CompletionRequest, config: &Config) -> Result<ContextData> {
    let mut context = ContextData::new();

    // Collect system information
    if config.context.include_system_info {
        context.system = system::collect_system_info(&request.session_id)?;
        debug!(
            "Gathered system info: {} {} ({})",
            context.system.os_type, context.system.os_version, context.system.arch
        );
    }

    // Gather history
    let history = history::read_history(&request.session_id, config.context.history_window)?;
    context.history = history;
    debug!("Gathered {} history entries", context.history.len());

    // Gather similar commands (if enabled and buffer is long enough)
    if config.context.similar_commands_enabled && request.buffer.len() >= 3 {
        let similar = history::find_similar_commands(
            &request.session_id,
            &request.buffer,
            config.context.similar_commands_window,
            config.context.similar_commands_max,
        )?;
        context.similar_commands = similar;
        debug!(
            "Gathered {} similar commands for query: {}",
            context.similar_commands.len(),
            request.buffer
        );
    }

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

    // System info: fixed cost (~40 tokens)
    total += 40;

    // History: count words × 1.3
    for cmd in &context.history {
        total += (cmd.split_whitespace().count() as f32 * 1.3).ceil() as usize;
    }

    // Similar commands: count words × 1.3
    for cmd in &context.similar_commands {
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

    // Define priority order (system info has highest implicit priority - never truncated)
    // Priority order for truncation (lowest to highest):
    // 1. Git plugins (40)
    // 2. CWD listing (60)
    // 3. Similar commands (70, implicit)
    // 4. History (80)

    // Truncate in order from lowest to highest priority
    while context.estimated_tokens > max_tokens {
        let before_tokens = context.estimated_tokens;

        // First: Remove git context (priority 40)
        if context.git.is_some() && priorities.plugins <= priorities.cwd_listing {
            context.git = None;
            context.estimated_tokens = estimate_tokens(context);
            if context.estimated_tokens != before_tokens {
                continue;
            }
        }

        // Second: Reduce CWD listing (priority 60)
        if !context.files.is_empty() && priorities.cwd_listing < priorities.history {
            let half = context.files.len() / 2;
            if half > 0 {
                context.files.truncate(half);
                context.estimated_tokens = estimate_tokens(context);
                if context.estimated_tokens != before_tokens {
                    continue;
                }
            }
        }

        // Third: Reduce similar commands (priority 70, between cwd and history)
        if !context.similar_commands.is_empty() {
            let half = context.similar_commands.len() / 2;
            if half > 0 {
                context.similar_commands.truncate(half);
                context.estimated_tokens = estimate_tokens(context);
                if context.estimated_tokens != before_tokens {
                    continue;
                }
            } else if context.similar_commands.len() == 1 {
                context.similar_commands.clear();
                context.estimated_tokens = estimate_tokens(context);
                if context.estimated_tokens != before_tokens {
                    continue;
                }
            }
        }

        // Fourth: Reduce history (priority 80)
        if !context.history.is_empty() {
            let half = context.history.len() / 2;
            if half > 0 {
                context.history = context.history.split_off(context.history.len() - half);
                context.estimated_tokens = estimate_tokens(context);
                if context.estimated_tokens != before_tokens {
                    continue;
                }
            }
        }

        // If nothing changed, break to avoid infinite loop
        if context.estimated_tokens == before_tokens {
            break;
        }
    }
}
