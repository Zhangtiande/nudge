use std::path::Path;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::NodePluginConfig;
use crate::daemon::context::plugin::{ContextPlugin, PluginContextData};

/// Node.js project context data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeContext {
    /// Package manager detected
    pub package_manager: PackageManager,
    /// Project name from package.json
    pub name: Option<String>,
    /// Project version from package.json
    pub version: Option<String>,
    /// Node version requirement (from .nvmrc, .node-version, or engines)
    pub node_version: Option<String>,
    /// Available npm scripts
    pub scripts: Vec<String>,
    /// Production dependencies
    pub dependencies: Vec<String>,
    /// Development dependencies
    pub dev_dependencies: Vec<String>,
    /// Whether this is a monorepo (has workspaces)
    pub is_monorepo: bool,
}

/// Package manager type
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    #[default]
    Unknown,
}

pub struct NodePlugin {
    config: NodePluginConfig,
}

impl NodePlugin {
    pub fn new(config: NodePluginConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ContextPlugin for NodePlugin {
    fn id(&self) -> &str {
        "node"
    }

    fn display_name(&self) -> &str {
        "Node.js"
    }

    fn is_applicable(&self, cwd: &Path) -> bool {
        cwd.join("package.json").exists()
    }

    async fn collect(&self, cwd: &Path) -> Result<PluginContextData> {
        let context = collect_node_context(cwd, &self.config).await?;
        let data = serde_json::to_value(&context).context("Failed to serialize node context")?;
        let priority = self.config.priority.unwrap_or(45);
        Ok(PluginContextData::new(self.id(), self.display_name(), data).with_priority(priority))
    }
}

/// Collect Node.js project context
async fn collect_node_context(cwd: &Path, config: &NodePluginConfig) -> Result<NodeContext> {
    let mut context = NodeContext::default();

    // Read package.json
    let pkg_path = cwd.join("package.json");
    let pkg_content = tokio::fs::read_to_string(&pkg_path)
        .await
        .context("Failed to read package.json")?;
    let pkg: Value = serde_json::from_str(&pkg_content).context("Failed to parse package.json")?;

    // Extract basic info
    context.name = pkg.get("name").and_then(|v| v.as_str()).map(String::from);
    context.version = pkg.get("version").and_then(|v| v.as_str()).map(String::from);

    // Detect package manager from lock files
    context.package_manager = detect_package_manager(cwd);

    // Extract Node version requirement
    context.node_version = detect_node_version(cwd, &pkg).await;

    // Extract scripts
    if let Some(scripts) = pkg.get("scripts").and_then(|v| v.as_object()) {
        context.scripts = scripts.keys().cloned().collect();
        context.scripts.sort();
    }

    // Extract dependencies (limited by max_dependencies)
    let max = config.max_dependencies;
    if let Some(deps) = pkg.get("dependencies").and_then(|v| v.as_object()) {
        context.dependencies = deps.keys().take(max).cloned().collect();
        context.dependencies.sort();
    }

    if let Some(dev_deps) = pkg.get("devDependencies").and_then(|v| v.as_object()) {
        context.dev_dependencies = dev_deps.keys().take(max).cloned().collect();
        context.dev_dependencies.sort();
    }

    // Check for monorepo (workspaces)
    context.is_monorepo = pkg.get("workspaces").is_some();

    Ok(context)
}

/// Detect package manager from lock files
fn detect_package_manager(cwd: &Path) -> PackageManager {
    if cwd.join("pnpm-lock.yaml").exists() {
        PackageManager::Pnpm
    } else if cwd.join("yarn.lock").exists() {
        PackageManager::Yarn
    } else if cwd.join("package-lock.json").exists() {
        PackageManager::Npm
    } else {
        PackageManager::Unknown
    }
}

/// Detect Node version from various sources
async fn detect_node_version(cwd: &Path, pkg: &Value) -> Option<String> {
    // Priority 1: .nvmrc
    if let Ok(content) = tokio::fs::read_to_string(cwd.join(".nvmrc")).await {
        let version = content.trim().to_string();
        if !version.is_empty() {
            return Some(version);
        }
    }

    // Priority 2: .node-version
    if let Ok(content) = tokio::fs::read_to_string(cwd.join(".node-version")).await {
        let version = content.trim().to_string();
        if !version.is_empty() {
            return Some(version);
        }
    }

    // Priority 3: package.json engines.node
    if let Some(engines) = pkg.get("engines") {
        if let Some(node_version) = engines.get("node").and_then(|v| v.as_str()) {
            return Some(node_version.to_string());
        }
    }

    None
}
