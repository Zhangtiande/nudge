use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Context plugin trait
#[async_trait]
pub trait ContextPlugin: Send + Sync {
    /// Plugin identifier
    fn id(&self) -> &str;

    /// Plugin display name
    fn display_name(&self) -> &str;

    /// Check if plugin is applicable for the given context
    fn is_applicable(&self, cwd: &Path) -> bool;

    /// Collect context data
    async fn collect(&self, cwd: &Path) -> Result<PluginContextData>;

    /// Get the default timeout for this plugin
    fn timeout(&self) -> Duration {
        Duration::from_millis(50)
    }

    /// Get the default priority for this plugin (1-100)
    fn default_priority(&self) -> u8 {
        40
    }
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
    pub fn new(plugin_id: &str, display_name: &str, data: Value) -> Self {
        Self {
            plugin_id: plugin_id.to_string(),
            display_name: display_name.to_string(),
            data,
            priority: 40,
            collection_time_ms: 0,
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

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
