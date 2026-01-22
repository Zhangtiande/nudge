use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Context plugin trait
#[async_trait]
#[allow(dead_code)]
pub trait ContextPlugin: Send + Sync {
    /// Plugin identifier
    fn id(&self) -> &str;

    /// Plugin display name
    fn display_name(&self) -> &str;

    /// Check if plugin is applicable for the given context
    fn is_applicable(&self, cwd: &Path) -> bool;

    /// Collect context data
    async fn collect(&self, cwd: &Path) -> Result<PluginContextData>;
}

/// Context data from a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContextData {
    /// Plugin identifier
    pub plugin_id: String,
    /// Plugin display name
    pub display_name: String,
    /// Key-value data collected by plugin
    pub data: Value,
    /// Priority for truncation (1-100)
    pub priority: u8,
    /// Time taken to collect (ms)
    pub collection_time_ms: u64,
}

impl PluginContextData {
    #[allow(dead_code)]
    pub fn new(plugin_id: &str, display_name: &str, data: Value) -> Self {
        Self {
            plugin_id: plugin_id.to_string(),
            display_name: display_name.to_string(),
            data,
            priority: 40,
            collection_time_ms: 0,
        }
    }

    #[allow(dead_code)]
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    #[allow(dead_code)]
    pub fn with_collection_time(mut self, ms: u64) -> Self {
        self.collection_time_ms = ms;
        self
    }
}

/// Plugin manifest for third-party plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique plugin identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Version string
    pub version: String,
    /// Description
    pub description: String,
    /// Required permissions
    pub permissions: Vec<Permission>,
    /// Maximum execution timeout (ms)
    pub timeout_ms: u64,
}

/// Permission types for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Permission {
    /// Read files in current working directory
    ReadCwd,
    /// Read files in home directory
    ReadHome,
    /// Read files at specific path
    ReadPath(String),
    /// Execute specific command
    ExecCommand(String),
}

// ========================================
// Plugin Activation Strategies
// ========================================

/// Plugin activation strategy trait
pub trait ActivationStrategy: Send + Sync {
    /// Check if plugin should activate based on context
    fn should_activate(&self, cwd: &Path, buffer: &str) -> bool;
}

/// Feature file-based activation (e.g., .git directory exists)
pub struct FeatureFileActivation {
    files: Vec<String>,
}

impl FeatureFileActivation {
    pub fn new(files: Vec<&str>) -> Self {
        Self {
            files: files.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl ActivationStrategy for FeatureFileActivation {
    fn should_activate(&self, cwd: &Path, _buffer: &str) -> bool {
        self.files.iter().any(|file| cwd.join(file).exists())
    }
}

/// Command prefix-based activation (e.g., buffer starts with "docker")
pub struct CommandPrefixActivation {
    prefixes: Vec<String>,
}

impl CommandPrefixActivation {
    pub fn new(prefixes: Vec<&str>) -> Self {
        Self {
            prefixes: prefixes.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl ActivationStrategy for CommandPrefixActivation {
    fn should_activate(&self, _cwd: &Path, buffer: &str) -> bool {
        let buffer_lower = buffer.trim().to_lowercase();
        self.prefixes.iter().any(|prefix| {
            buffer_lower == prefix.as_str()
                || buffer_lower.starts_with(&format!("{} ", prefix))
                || buffer_lower.starts_with(&format!("{}\t", prefix))
        })
    }
}

/// Combined activation (OR logic: any strategy matches)
pub struct CombinedActivation {
    strategies: Vec<Box<dyn ActivationStrategy>>,
}

impl CombinedActivation {
    pub fn new(strategies: Vec<Box<dyn ActivationStrategy>>) -> Self {
        Self { strategies }
    }
}

impl ActivationStrategy for CombinedActivation {
    fn should_activate(&self, cwd: &Path, buffer: &str) -> bool {
        self.strategies
            .iter()
            .any(|s| s.should_activate(cwd, buffer))
    }
}

// ========================================
// Plugin Manager
// ========================================

use std::time::Instant;
use tokio::time::timeout;
use tracing::{debug, warn};

/// Plugin registration entry
pub struct PluginRegistration {
    pub plugin: Box<dyn ContextPlugin>,
    pub activation: Box<dyn ActivationStrategy>,
    pub enabled: bool,
    pub timeout_ms: u64,
    pub priority: u8,
}

/// Plugin manager - coordinates plugin lifecycle
pub struct PluginManager {
    plugins: Vec<PluginRegistration>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register a plugin with activation strategy
    pub fn register(
        mut self,
        plugin: Box<dyn ContextPlugin>,
        activation: Box<dyn ActivationStrategy>,
        enabled: bool,
        timeout_ms: u64,
        priority: u8,
    ) -> Self {
        self.plugins.push(PluginRegistration {
            plugin,
            activation,
            enabled,
            timeout_ms,
            priority,
        });
        self
    }

    /// Collect context from all activated plugins
    pub async fn collect_all(&self, cwd: &Path, buffer: &str) -> Vec<PluginContextData> {
        let mut results = Vec::new();

        for registration in &self.plugins {
            if !registration.enabled {
                continue;
            }

            if !registration.activation.should_activate(cwd, buffer) {
                debug!(
                    "Plugin '{}' not activated (condition not met)",
                    registration.plugin.id()
                );
                continue;
            }

            let plugin_id = registration.plugin.id().to_string();
            let timeout_duration = Duration::from_millis(registration.timeout_ms);
            let priority = registration.priority;

            let start = Instant::now();

            match timeout(timeout_duration, registration.plugin.collect(cwd)).await {
                Ok(Ok(mut data)) => {
                    data.priority = priority;
                    data.collection_time_ms = start.elapsed().as_millis() as u64;
                    debug!(
                        "Plugin '{}' collected in {}ms",
                        plugin_id, data.collection_time_ms
                    );
                    results.push(data);
                }
                Ok(Err(e)) => {
                    debug!("Plugin '{}' collection failed: {}", plugin_id, e);
                }
                Err(_) => {
                    warn!(
                        "Plugin '{}' timed out after {}ms",
                        plugin_id,
                        timeout_duration.as_millis()
                    );
                }
            }
        }

        results
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
