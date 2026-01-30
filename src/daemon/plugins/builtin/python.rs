use std::path::Path;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use toml::Value;

use crate::config::PythonPluginConfig;
use crate::daemon::context::plugin::{ContextPlugin, PluginContextData};

/// Python project context data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PythonContext {
    /// Package manager detected
    pub package_manager: PythonPackageManager,
    /// Project name from pyproject.toml
    pub name: Option<String>,
    /// Project version from pyproject.toml
    pub version: Option<String>,
    /// Python version requirement (requires-python)
    pub python_version: Option<String>,
    /// Dependencies
    pub dependencies: Vec<String>,
    /// Development dependencies
    pub dev_dependencies: Vec<String>,
    /// Available scripts/entry points
    pub scripts: Vec<String>,
}

/// Python package manager type
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PythonPackageManager {
    Uv,
    Poetry,
    Pip,
    #[default]
    Unknown,
}

pub struct PythonPlugin {
    config: PythonPluginConfig,
}

impl PythonPlugin {
    pub fn new(config: PythonPluginConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ContextPlugin for PythonPlugin {
    fn id(&self) -> &str {
        "python"
    }

    fn display_name(&self) -> &str {
        "Python"
    }

    fn is_applicable(&self, cwd: &Path) -> bool {
        cwd.join("pyproject.toml").exists()
            || cwd.join("uv.lock").exists()
            || cwd.join("requirements.txt").exists()
    }

    async fn collect(&self, cwd: &Path) -> Result<PluginContextData> {
        let context = collect_python_context(cwd, &self.config).await?;
        let data = serde_json::to_value(&context).context("Failed to serialize python context")?;
        let priority = self.config.priority.unwrap_or(45);
        Ok(PluginContextData::new(self.id(), self.display_name(), data).with_priority(priority))
    }
}

/// Collect Python project context
async fn collect_python_context(cwd: &Path, config: &PythonPluginConfig) -> Result<PythonContext> {
    let mut context = PythonContext {
        package_manager: detect_python_package_manager(cwd),
        ..Default::default()
    };

    // Try to read pyproject.toml first
    let pyproject_path = cwd.join("pyproject.toml");
    if pyproject_path.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&pyproject_path).await {
            if let Ok(pyproject) = toml::from_str::<Value>(&content) {
                parse_pyproject(&mut context, &pyproject, config.max_dependencies);
            }
        }
    }

    // Fallback: read requirements.txt for dependencies
    if context.dependencies.is_empty() {
        if let Ok(content) = tokio::fs::read_to_string(cwd.join("requirements.txt")).await {
            context.dependencies = parse_requirements(&content, config.max_dependencies);
        }
    }

    // Read dev requirements if exists
    if context.dev_dependencies.is_empty() {
        for dev_file in [
            "requirements-dev.txt",
            "requirements_dev.txt",
            "dev-requirements.txt",
        ] {
            if let Ok(content) = tokio::fs::read_to_string(cwd.join(dev_file)).await {
                context.dev_dependencies = parse_requirements(&content, config.max_dependencies);
                break;
            }
        }
    }

    Ok(context)
}

/// Detect Python package manager from lock files
fn detect_python_package_manager(cwd: &Path) -> PythonPackageManager {
    if cwd.join("uv.lock").exists() {
        PythonPackageManager::Uv
    } else if cwd.join("poetry.lock").exists() {
        PythonPackageManager::Poetry
    } else if cwd.join("requirements.txt").exists() {
        PythonPackageManager::Pip
    } else {
        PythonPackageManager::Unknown
    }
}

/// Parse pyproject.toml for project info
fn parse_pyproject(context: &mut PythonContext, pyproject: &Value, max_deps: usize) {
    // PEP 621 standard: [project] section
    if let Some(project) = pyproject.get("project") {
        context.name = project
            .get("name")
            .and_then(|v| v.as_str())
            .map(String::from);
        context.version = project
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from);
        context.python_version = project
            .get("requires-python")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Extract dependencies
        if let Some(deps) = project.get("dependencies").and_then(|v| v.as_array()) {
            context.dependencies = deps
                .iter()
                .filter_map(|d| d.as_str())
                .map(extract_package_name)
                .take(max_deps)
                .collect();
            context.dependencies.sort();
        }

        // Extract optional dependencies (often used for dev deps)
        if let Some(optional) = project
            .get("optional-dependencies")
            .and_then(|v| v.as_table())
        {
            if let Some(dev) = optional.get("dev").and_then(|v| v.as_array()) {
                context.dev_dependencies = dev
                    .iter()
                    .filter_map(|d| d.as_str())
                    .map(extract_package_name)
                    .take(max_deps)
                    .collect();
                context.dev_dependencies.sort();
            }
        }

        // Extract scripts
        if let Some(scripts) = project.get("scripts").and_then(|v| v.as_table()) {
            context.scripts = scripts.keys().cloned().collect();
            context.scripts.sort();
        }
    }

    // Poetry-specific: [tool.poetry] section
    if let Some(tool) = pyproject.get("tool") {
        if let Some(poetry) = tool.get("poetry") {
            if context.name.is_none() {
                context.name = poetry
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(String::from);
            }
            if context.version.is_none() {
                context.version = poetry
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(String::from);
            }

            // Poetry dependencies
            if context.dependencies.is_empty() {
                if let Some(deps) = poetry.get("dependencies").and_then(|v| v.as_table()) {
                    context.dependencies = deps
                        .keys()
                        .filter(|k| *k != "python")
                        .take(max_deps)
                        .cloned()
                        .collect();
                    context.dependencies.sort();
                }
            }

            // Poetry dev dependencies (group.dev.dependencies)
            if context.dev_dependencies.is_empty() {
                if let Some(group) = poetry.get("group") {
                    if let Some(dev) = group.get("dev") {
                        if let Some(deps) = dev.get("dependencies").and_then(|v| v.as_table()) {
                            context.dev_dependencies =
                                deps.keys().take(max_deps).cloned().collect();
                            context.dev_dependencies.sort();
                        }
                    }
                }
            }

            // Poetry scripts
            if context.scripts.is_empty() {
                if let Some(scripts) = poetry.get("scripts").and_then(|v| v.as_table()) {
                    context.scripts = scripts.keys().cloned().collect();
                    context.scripts.sort();
                }
            }
        }
    }
}

/// Parse requirements.txt format
fn parse_requirements(content: &str, max_deps: usize) -> Vec<String> {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with('-')
        })
        .map(|line| extract_package_name(line.trim()))
        .take(max_deps)
        .collect()
}

/// Extract package name from dependency string (e.g., "requests>=2.28.0" -> "requests")
fn extract_package_name(dep: &str) -> String {
    // Find first occurrence of version specifier characters
    let name_end = dep
        .find(['>', '<', '=', '!', '~', '[', ';'])
        .unwrap_or(dep.len());
    dep[..name_end].trim().to_string()
}
