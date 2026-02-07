use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::config::DockerPluginConfig;
use crate::daemon::context::plugin::{ContextPlugin, PluginContextData};

/// Docker context data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DockerContext {
    /// Has docker-compose.yml
    pub has_compose: bool,
    /// Docker compose service names
    pub compose_services: Vec<String>,
    /// Running containers (if show_containers enabled)
    pub running_containers: Vec<ContainerInfo>,
    /// Container count
    pub container_count: usize,
    /// Recent images
    pub recent_images: Vec<String>,
    /// Docker daemon available
    pub daemon_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub status: String,
}

/// Docker plugin timeout (100ms - slightly longer than git due to daemon communication)
#[allow(dead_code)]
const DOCKER_TIMEOUT_MS: u64 = 100;

pub struct DockerPlugin {
    config: DockerPluginConfig,
}

impl DockerPlugin {
    pub fn new(config: DockerPluginConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ContextPlugin for DockerPlugin {
    fn id(&self) -> &str {
        "docker"
    }

    fn display_name(&self) -> &str {
        "Docker"
    }

    fn is_applicable(&self, cwd: &Path) -> bool {
        // Check for Docker feature files
        cwd.join("Dockerfile").exists()
            || cwd.join("docker-compose.yml").exists()
            || cwd.join("docker-compose.yaml").exists()
            || cwd.join("compose.yml").exists()
            || cwd.join("compose.yaml").exists()
    }

    async fn collect(&self, cwd: &Path) -> Result<PluginContextData> {
        let context = collect_docker_context(cwd, &self.config).await?;

        let data = serde_json::to_value(&context).context("Failed to serialize docker context")?;

        let priority = self.config.priority.unwrap_or(45);

        Ok(PluginContextData::new(self.id(), self.display_name(), data).with_priority(priority))
    }
}

/// Collect Docker context (MVP version)
async fn collect_docker_context(cwd: &Path, config: &DockerPluginConfig) -> Result<DockerContext> {
    let mut context = DockerContext::default();

    // Check for docker-compose files and extract services
    if config.include_compose {
        if let Some(services) = read_compose_services(cwd) {
            context.has_compose = true;
            context.compose_services = services;
        }
    }

    // Check if Docker daemon is available
    context.daemon_available = check_docker_daemon().await;

    if context.daemon_available {
        // Get running containers if enabled
        if config.show_containers {
            if let Some(containers) = get_running_containers(config.max_containers).await {
                context.container_count = containers.len();
                context.running_containers = containers;
            }
        }

        // Get recent images
        if let Some(images) = get_recent_images(config.max_images).await {
            context.recent_images = images;
        }
    }

    Ok(context)
}

/// Check if Docker daemon is available
async fn check_docker_daemon() -> bool {
    tokio::task::spawn_blocking(|| {
        Command::new("docker")
            .arg("info")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
    .await
    .unwrap_or(false)
}

/// Read docker-compose and extract service names
fn read_compose_services(cwd: &Path) -> Option<Vec<String>> {
    let compose_paths = vec![
        cwd.join("docker-compose.yml"),
        cwd.join("docker-compose.yaml"),
        cwd.join("compose.yml"),
        cwd.join("compose.yaml"),
    ];

    for path in compose_paths {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let services = extract_services_from_yaml(&content);
            if !services.is_empty() {
                return Some(services);
            }
        }
    }

    None
}

/// Minimal docker-compose structure for parsing service names
#[derive(Debug, Deserialize)]
struct ComposeFile {
    services: Option<HashMap<String, serde_yaml::Value>>,
}

/// Extract service names from docker-compose YAML
fn extract_services_from_yaml(content: &str) -> Vec<String> {
    // Try to parse as YAML
    match serde_yaml::from_str::<ComposeFile>(content) {
        Ok(compose) => {
            if let Some(services) = compose.services {
                let mut service_names: Vec<String> = services.keys().cloned().collect();
                service_names.sort();
                service_names
            } else {
                Vec::new()
            }
        }
        Err(_) => {
            // If parsing fails, return empty vec (graceful degradation)
            Vec::new()
        }
    }
}

/// Get running Docker containers
async fn get_running_containers(max: usize) -> Option<Vec<ContainerInfo>> {
    tokio::task::spawn_blocking(move || {
        let output = Command::new("docker")
            .args(["ps", "--format", "{{.ID}}|{{.Names}}|{{.Status}}"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let containers: Vec<ContainerInfo> = stdout
            .lines()
            .filter(|l| !l.is_empty())
            .take(max)
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 3 {
                    Some(ContainerInfo {
                        id: parts[0].to_string(),
                        name: parts[1].to_string(),
                        status: parts[2].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        if containers.is_empty() {
            None
        } else {
            Some(containers)
        }
    })
    .await
    .ok()?
}

/// Get recent Docker images
async fn get_recent_images(max: usize) -> Option<Vec<String>> {
    tokio::task::spawn_blocking(move || {
        let output = Command::new("docker")
            .args(["images", "--format", "{{.Repository}}:{{.Tag}}"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let images: Vec<String> = stdout
            .lines()
            .filter(|l| !l.is_empty() && !l.contains("<none>"))
            .take(max)
            .map(|s| s.to_string())
            .collect();

        if images.is_empty() {
            None
        } else {
            Some(images)
        }
    })
    .await
    .ok()?
}
