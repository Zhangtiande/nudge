pub mod cwd;
pub mod history;
pub mod plugin;
pub mod system;

use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use super::plugins::builtin::git::GitContext;
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
    /// Git context (if applicable) - legacy field for backward compatibility
    pub git: Option<GitContext>,
    /// System information
    pub system: SystemInfo,
    /// Plugin context data (new unified field)
    #[serde(default)]
    pub plugins: HashMap<String, Value>,
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
            plugins: HashMap::new(),
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

    // Gather plugin context using PluginManager
    let plugin_manager = create_plugin_manager(config);
    let plugin_data = plugin_manager
        .collect_all(&request.cwd, &request.buffer)
        .await;

    // Populate plugins HashMap and maintain legacy git field
    for data in plugin_data {
        let plugin_id = data.plugin_id.clone();
        let value = data.data.clone();

        // Store in plugins map
        context.plugins.insert(plugin_id.clone(), value.clone());

        // Legacy: populate git field for backward compatibility
        if plugin_id == "git" {
            if let Ok(git_ctx) = serde_json::from_value::<GitContext>(value) {
                context.git = Some(git_ctx);
            }
        }

        debug!("Gathered {} plugin context", plugin_id);
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

    // Legacy git context: estimate based on content
    if let Some(git) = &context.git {
        if git.branch.is_some() {
            total += 5;
        }
        total += git.staged.len();
        total += git.recent_commits.len() * 10;
    }

    // Plugin context: estimate based on JSON size (rough approximation)
    for (_plugin_id, data) in &context.plugins {
        // Conservative estimate: JSON string length / 4 characters per token
        let json_str = serde_json::to_string(data).unwrap_or_default();
        total += json_str.len() / 4;
    }

    total
}

/// Truncate context by priority
fn truncate_by_priority(context: &mut ContextData, config: &Config) {
    let priorities = &config.context.priorities;
    let max_tokens = config.context.max_total_tokens;

    // Define priority order (system info has highest implicit priority - never truncated)
    // Priority order for truncation (lowest to highest):
    // 1. Plugin contexts (40-50 range, configurable)
    // 2. CWD listing (60)
    // 3. Similar commands (70, implicit)
    // 4. History (80)

    // Truncate in order from lowest to highest priority
    while context.estimated_tokens > max_tokens {
        let before_tokens = context.estimated_tokens;

        // First: Remove plugin contexts (priority ~40-50, lowest)
        if !context.plugins.is_empty() && priorities.plugins <= priorities.cwd_listing {
            // Simple MVP: clear all plugins at once
            // Future: could sort by priority and remove lowest first
            context.plugins.clear();
            context.git = None; // Also clear legacy git field
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

/// Create and configure plugin manager with registered plugins
fn create_plugin_manager(config: &Config) -> plugin::PluginManager {
    use super::plugins::builtin::git::GitPlugin;
    use super::plugins::community::docker::DockerPlugin;
    use plugin::{
        CombinedActivation, CommandPrefixActivation, FeatureFileActivation, PluginManager,
    };

    PluginManager::new()
        // Register Git plugin
        .register(
            Box::new(GitPlugin::new(config.plugins.git.clone())),
            Box::new(CombinedActivation::new(vec![
                Box::new(FeatureFileActivation::new(vec![".git"])),
                Box::new(CommandPrefixActivation::new(vec!["git"])),
            ])),
            config.plugins.git.enabled,
            50, // Git timeout is hardcoded to 50ms
            config.plugins.git.priority.unwrap_or(50),
        )
        // Register Docker plugin
        .register(
            Box::new(DockerPlugin::new(config.plugins.docker.clone())),
            Box::new(CombinedActivation::new(vec![
                Box::new(FeatureFileActivation::new(vec![
                    "Dockerfile",
                    "docker-compose.yml",
                    "docker-compose.yaml",
                    "compose.yml",
                    "compose.yaml",
                ])),
                Box::new(CommandPrefixActivation::new(vec![
                    "docker",
                    "docker-compose",
                ])),
            ])),
            config.plugins.docker.enabled,
            config.plugins.docker.timeout_ms,
            config.plugins.docker.priority.unwrap_or(45),
        )
}
