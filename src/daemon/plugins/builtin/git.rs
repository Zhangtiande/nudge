use std::path::Path;
use std::process::Command;
use std::time::Instant;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::config::GitPluginConfig;

/// Git context data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitContext {
    /// Configured depth level used
    pub depth: GitDepth,
    /// Current branch name
    pub branch: Option<String>,
    /// Local branch names (for switch/checkout completion)
    pub local_branches: Vec<String>,
    /// Repository status
    pub status: GitStatus,
    /// Staged files
    pub staged: Vec<String>,
    /// Unstaged files (detailed depth only)
    pub unstaged: Vec<String>,
}

/// Git depth level
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitDepth {
    Light,
    #[default]
    Standard,
    Detailed,
}

/// Git repository status
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitStatus {
    Clean,
    Dirty,
    #[default]
    Unknown,
}

/// Strict timeout for git operations (50ms)
#[allow(dead_code)]
const GIT_TIMEOUT_MS: u64 = 50;

/// Collect git context
pub async fn collect(cwd: &Path, config: &GitPluginConfig) -> Result<GitContext> {
    // Check if this is a git repository
    if !is_git_repo(cwd) {
        anyhow::bail!("Not a git repository");
    }

    let depth: GitDepth = config.depth.into();
    let start = Instant::now();

    let context = collect_git_context(cwd, depth, config.max_branches).await?;
    debug!("Git context collected in {:?}", start.elapsed());
    Ok(context)
}

/// Check if directory is inside a git repository
fn is_git_repo(cwd: &Path) -> bool {
    // Quick check for .git directory
    if cwd.join(".git").exists() {
        return true;
    }

    // Fallback to git command (handles worktrees, submodules)
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(cwd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Collect git context data
async fn collect_git_context(
    cwd: &Path,
    depth: GitDepth,
    max_branches: usize,
) -> Result<GitContext> {
    let mut context = GitContext {
        depth,
        ..Default::default()
    };

    // Always get branch and status (light)
    context.branch = get_branch(cwd).await;
    context.status = get_status(cwd).await;

    // Standard and detailed: get staged files and local branch list
    if matches!(depth, GitDepth::Standard | GitDepth::Detailed) {
        context.staged = get_staged_files(cwd).await;
        context.local_branches = get_local_branches(cwd, max_branches).await;
        if let Some(current) = &context.branch {
            if let Some(pos) = context.local_branches.iter().position(|b| b == current) {
                if pos != 0 {
                    let current_branch = context.local_branches.remove(pos);
                    context.local_branches.insert(0, current_branch);
                }
            }
        }
    }

    // Detailed only: get unstaged files
    if depth == GitDepth::Detailed {
        context.unstaged = get_unstaged_files(cwd).await;
    }

    Ok(context)
}

/// Get current branch name
async fn get_branch(cwd: &Path) -> Option<String> {
    let cwd = cwd.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&cwd)
            .output()
            .ok()?;

        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if branch.is_empty() {
                // Detached HEAD
                None
            } else {
                Some(branch)
            }
        } else {
            None
        }
    })
    .await
    .ok()?
}

/// Get repository status (clean/dirty)
async fn get_status(cwd: &Path) -> GitStatus {
    let cwd = cwd.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&cwd)
            .output();

        match output {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                if stdout.trim().is_empty() {
                    GitStatus::Clean
                } else {
                    GitStatus::Dirty
                }
            }
            _ => GitStatus::Unknown,
        }
    })
    .await
    .unwrap_or(GitStatus::Unknown)
}

/// Get staged files
async fn get_staged_files(cwd: &Path) -> Vec<String> {
    let cwd = cwd.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let output = Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .current_dir(&cwd)
            .output();

        match output {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect(),
            _ => Vec::new(),
        }
    })
    .await
    .unwrap_or_default()
}

/// Get unstaged files
async fn get_unstaged_files(cwd: &Path) -> Vec<String> {
    let cwd = cwd.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let output = Command::new("git")
            .args(["diff", "--name-only"])
            .current_dir(&cwd)
            .output();

        match output {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect(),
            _ => Vec::new(),
        }
    })
    .await
    .unwrap_or_default()
}

/// Get local branch names
async fn get_local_branches(cwd: &Path, max: usize) -> Vec<String> {
    let cwd = cwd.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let output = Command::new("git")
            .args(["for-each-ref", "--format=%(refname:short)", "refs/heads"])
            .current_dir(&cwd)
            .output();

        match output {
            Ok(o) if o.status.success() => {
                let mut branches: Vec<String> = String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .map(|l| l.to_string())
                    .collect();
                branches.sort();
                branches.truncate(max);
                branches
            }
            _ => Vec::new(),
        }
    })
    .await
    .unwrap_or_default()
}

impl From<crate::config::GitDepth> for GitDepth {
    fn from(depth: crate::config::GitDepth) -> Self {
        match depth {
            crate::config::GitDepth::Light => GitDepth::Light,
            crate::config::GitDepth::Standard => GitDepth::Standard,
            crate::config::GitDepth::Detailed => GitDepth::Detailed,
        }
    }
}

// ========================================
// Plugin Trait Implementation
// ========================================

use crate::daemon::context::plugin::{ContextPlugin, PluginContextData};
use async_trait::async_trait;

/// Git context plugin
pub struct GitPlugin {
    config: GitPluginConfig,
}

impl GitPlugin {
    pub fn new(config: GitPluginConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ContextPlugin for GitPlugin {
    fn id(&self) -> &str {
        "git"
    }

    fn display_name(&self) -> &str {
        "Git Repository"
    }

    fn is_applicable(&self, cwd: &Path) -> bool {
        is_git_repo(cwd)
    }

    async fn collect(&self, cwd: &Path) -> Result<PluginContextData> {
        // Call existing collect function
        let git_context = collect(cwd, &self.config).await?;

        // Serialize to JSON
        let data = serde_json::to_value(&git_context)?;

        let priority = self.config.priority.unwrap_or(50);

        Ok(PluginContextData::new(self.id(), self.display_name(), data).with_priority(priority))
    }
}
