use std::path::Path;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use toml::Value;

use crate::config::RustPluginConfig;
use crate::daemon::context::plugin::{ContextPlugin, PluginContextData};

/// Rust project context data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RustContext {
    /// Project name from Cargo.toml
    pub name: Option<String>,
    /// Project version from Cargo.toml
    pub version: Option<String>,
    /// Rust version requirement (rust-version field)
    pub rust_version: Option<String>,
    /// Whether this is a workspace
    pub is_workspace: bool,
    /// Workspace members (if workspace)
    pub workspace_members: Vec<String>,
    /// Binary targets
    pub binaries: Vec<String>,
}

pub struct RustPlugin {
    config: RustPluginConfig,
}

impl RustPlugin {
    pub fn new(config: RustPluginConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ContextPlugin for RustPlugin {
    fn id(&self) -> &str {
        "rust"
    }

    fn display_name(&self) -> &str {
        "Rust"
    }

    fn is_applicable(&self, cwd: &Path) -> bool {
        cwd.join("Cargo.toml").exists()
    }

    async fn collect(&self, cwd: &Path) -> Result<PluginContextData> {
        let context = collect_rust_context(cwd, &self.config).await?;
        let data = serde_json::to_value(&context).context("Failed to serialize rust context")?;
        let priority = self.config.priority.unwrap_or(45);
        Ok(PluginContextData::new(self.id(), self.display_name(), data).with_priority(priority))
    }
}

/// Collect Rust project context
async fn collect_rust_context(cwd: &Path, _config: &RustPluginConfig) -> Result<RustContext> {
    let mut context = RustContext::default();

    // Read Cargo.toml
    let cargo_path = cwd.join("Cargo.toml");
    let cargo_content = tokio::fs::read_to_string(&cargo_path)
        .await
        .context("Failed to read Cargo.toml")?;
    let cargo: Value = toml::from_str(&cargo_content).context("Failed to parse Cargo.toml")?;

    // Check if this is a workspace
    if let Some(workspace) = cargo.get("workspace") {
        context.is_workspace = true;

        // Extract workspace members
        if let Some(members) = workspace.get("members").and_then(|v| v.as_array()) {
            context.workspace_members = members
                .iter()
                .filter_map(|m| m.as_str())
                .map(String::from)
                .collect();
        }
    }

    // Extract package info (may not exist in workspace root)
    if let Some(package) = cargo.get("package") {
        context.name = package
            .get("name")
            .and_then(|v| v.as_str())
            .map(String::from);
        context.version = package
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from);
        context.rust_version = package
            .get("rust-version")
            .and_then(|v| v.as_str())
            .map(String::from);
    }

    // Extract binary targets
    if let Some(bins) = cargo.get("bin").and_then(|v| v.as_array()) {
        context.binaries = bins
            .iter()
            .filter_map(|b| b.get("name").and_then(|n| n.as_str()))
            .map(String::from)
            .collect();
    }

    // If no explicit [[bin]], check for default binary (same as package name)
    if context.binaries.is_empty() && context.name.is_some() {
        // Check if src/main.rs exists (default binary)
        if cwd.join("src/main.rs").exists() {
            if let Some(name) = &context.name {
                context.binaries.push(name.clone());
            }
        }
    }

    Ok(context)
}
