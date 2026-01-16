use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::config::GitPluginConfig;

/// Git context data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitContext {
    /// Configured depth level used
    pub depth: GitDepth,
    /// Current branch name
    pub branch: Option<String>,
    /// Repository status
    pub status: GitStatus,
    /// Staged files
    pub staged: Vec<String>,
    /// Unstaged files (detailed depth only)
    pub unstaged: Vec<String>,
    /// Recent commit messages
    pub recent_commits: Vec<String>,
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
const GIT_TIMEOUT_MS: u64 = 50;

/// Collect git context
pub async fn collect(cwd: &Path, config: &GitPluginConfig) -> Result<GitContext> {
    // Check if this is a git repository
    if !is_git_repo(cwd) {
        anyhow::bail!("Not a git repository");
    }

    let depth: GitDepth = config.depth.into();
    let start = Instant::now();

    // Use timeout wrapper for all git operations
    let result = timeout(
        Duration::from_millis(GIT_TIMEOUT_MS),
        collect_git_context(cwd, depth, config.recent_commits),
    )
    .await;

    match result {
        Ok(Ok(context)) => {
            debug!("Git context collected in {:?}", start.elapsed());
            Ok(context)
        }
        Ok(Err(e)) => {
            warn!("Git context error: {}", e);
            Err(e)
        }
        Err(_) => {
            warn!("Git context timed out after {}ms", GIT_TIMEOUT_MS);
            anyhow::bail!("Git operations timed out");
        }
    }
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
    recent_commits_count: usize,
) -> Result<GitContext> {
    let mut context = GitContext {
        depth,
        ..Default::default()
    };

    // Always get branch and status (light)
    context.branch = get_branch(cwd);
    context.status = get_status(cwd);

    // Standard and detailed: get staged files and commits
    if matches!(depth, GitDepth::Standard | GitDepth::Detailed) {
        context.staged = get_staged_files(cwd);
        context.recent_commits = get_recent_commits(cwd, recent_commits_count);
    }

    // Detailed only: get unstaged files
    if depth == GitDepth::Detailed {
        context.unstaged = get_unstaged_files(cwd);
    }

    Ok(context)
}

/// Get current branch name
fn get_branch(cwd: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(cwd)
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
}

/// Get repository status (clean/dirty)
fn get_status(cwd: &Path) -> GitStatus {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(cwd)
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
}

/// Get staged files
fn get_staged_files(cwd: &Path) -> Vec<String> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(cwd)
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

/// Get unstaged files
fn get_unstaged_files(cwd: &Path) -> Vec<String> {
    let output = Command::new("git")
        .args(["diff", "--name-only"])
        .current_dir(cwd)
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

/// Get recent commits
fn get_recent_commits(cwd: &Path, count: usize) -> Vec<String> {
    let output = Command::new("git")
        .args(["log", "--oneline", &format!("-{}", count)])
        .current_dir(cwd)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| {
                    // Extract just the commit message (remove hash)
                    l.splitn(2, ' ').nth(1).unwrap_or(l).to_string()
                })
                .collect()
        }
        _ => Vec::new(),
    }
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
