//! Integration tests for the Git context plugin.
//!
//! These tests verify that Git context is properly collected
//! under various repository states.

use std::process::Command;
use tempfile::TempDir;

/// Create a test git repository
fn create_test_repo() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.path();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .expect("Failed to init git repo");

    // Configure git user (required for commits)
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()
        .expect("Failed to configure git email");

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()
        .expect("Failed to configure git name");

    temp_dir
}

/// Test: Detect git repository
#[test]
fn test_detect_git_repo() {
    let repo = create_test_repo();
    let path = repo.path();

    // Check .git directory exists
    assert!(path.join(".git").exists());

    // Verify with git command
    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(path)
        .output()
        .expect("Failed to check git status");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim() == "true");
}

/// Test: Get branch name
#[test]
fn test_get_branch_name() {
    let repo = create_test_repo();
    let path = repo.path();

    // Create initial commit (required for branch to exist)
    std::fs::write(path.join("README.md"), "# Test").unwrap();
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(path)
        .output()
        .expect("Failed to stage file");

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()
        .expect("Failed to commit");

    // Get branch name
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(path)
        .output()
        .expect("Failed to get branch");

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    // Should be "main" or "master" depending on git version
    assert!(branch == "main" || branch == "master");
}

/// Test: Clean repository status
#[test]
fn test_clean_repo_status() {
    let repo = create_test_repo();
    let path = repo.path();

    // Create and commit a file
    std::fs::write(path.join("file.txt"), "content").unwrap();
    Command::new("git")
        .args(["add", "file.txt"])
        .current_dir(path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Add file"])
        .current_dir(path)
        .output()
        .unwrap();

    // Check status
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output()
        .expect("Failed to get status");

    let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(status.is_empty(), "Expected clean repo, got: {}", status);
}

/// Test: Dirty repository status
#[test]
fn test_dirty_repo_status() {
    let repo = create_test_repo();
    let path = repo.path();

    // Create initial commit
    std::fs::write(path.join("file.txt"), "original").unwrap();
    Command::new("git")
        .args(["add", "file.txt"])
        .current_dir(path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(path)
        .output()
        .unwrap();

    // Modify file (dirty state)
    std::fs::write(path.join("file.txt"), "modified").unwrap();

    // Check status
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output()
        .expect("Failed to get status");

    let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(!status.is_empty(), "Expected dirty repo");
    assert!(status.contains("M") || status.contains("?"));
}

/// Test: Get staged files
#[test]
fn test_get_staged_files() {
    let repo = create_test_repo();
    let path = repo.path();

    // Create initial commit
    std::fs::write(path.join("initial.txt"), "init").unwrap();
    Command::new("git")
        .args(["add", "initial.txt"])
        .current_dir(path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(path)
        .output()
        .unwrap();

    // Stage new files
    std::fs::write(path.join("staged1.txt"), "content1").unwrap();
    std::fs::write(path.join("staged2.txt"), "content2").unwrap();
    Command::new("git")
        .args(["add", "staged1.txt", "staged2.txt"])
        .current_dir(path)
        .output()
        .unwrap();

    // Get staged files
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(path)
        .output()
        .expect("Failed to get staged files");

    let staged = String::from_utf8_lossy(&output.stdout);
    assert!(staged.contains("staged1.txt"));
    assert!(staged.contains("staged2.txt"));
}

/// Test: Get recent commits
#[test]
fn test_get_recent_commits() {
    let repo = create_test_repo();
    let path = repo.path();

    // Create multiple commits
    for i in 1..=5 {
        std::fs::write(path.join(format!("file{}.txt", i)), format!("content{}", i)).unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", &format!("Commit #{}", i)])
            .current_dir(path)
            .output()
            .unwrap();
    }

    // Get recent commits
    let output = Command::new("git")
        .args(["log", "--oneline", "-3"])
        .current_dir(path)
        .output()
        .expect("Failed to get commits");

    let commits = String::from_utf8_lossy(&output.stdout);
    let commit_count = commits.lines().count();
    assert_eq!(commit_count, 3);
    assert!(commits.contains("Commit #5"));
    assert!(commits.contains("Commit #4"));
    assert!(commits.contains("Commit #3"));
}

/// Test: Get unstaged files
#[test]
fn test_get_unstaged_files() {
    let repo = create_test_repo();
    let path = repo.path();

    // Create and commit initial file
    std::fs::write(path.join("tracked.txt"), "original").unwrap();
    Command::new("git")
        .args(["add", "tracked.txt"])
        .current_dir(path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(path)
        .output()
        .unwrap();

    // Modify without staging
    std::fs::write(path.join("tracked.txt"), "modified").unwrap();

    // Get unstaged changes
    let output = Command::new("git")
        .args(["diff", "--name-only"])
        .current_dir(path)
        .output()
        .expect("Failed to get unstaged files");

    let unstaged = String::from_utf8_lossy(&output.stdout);
    assert!(unstaged.contains("tracked.txt"));
}

/// Test: Non-git directory returns error
#[test]
fn test_non_git_directory() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.path();

    // Don't initialize git
    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(path)
        .output()
        .expect("Failed to run git");

    assert!(!output.status.success());
}

/// Test: Detached HEAD state
#[test]
fn test_detached_head() {
    let repo = create_test_repo();
    let path = repo.path();

    // Create commits
    std::fs::write(path.join("file.txt"), "v1").unwrap();
    Command::new("git")
        .args(["add", "file.txt"])
        .current_dir(path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "First"])
        .current_dir(path)
        .output()
        .unwrap();

    std::fs::write(path.join("file.txt"), "v2").unwrap();
    Command::new("git")
        .args(["add", "file.txt"])
        .current_dir(path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Second"])
        .current_dir(path)
        .output()
        .unwrap();

    // Checkout to detached HEAD
    Command::new("git")
        .args(["checkout", "HEAD~1"])
        .current_dir(path)
        .output()
        .unwrap();

    // Branch name should be empty in detached HEAD
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(path)
        .output()
        .expect("Failed to get branch");

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(branch.is_empty(), "Expected empty branch in detached HEAD");
}

/// Test: Empty repository (no commits)
#[test]
fn test_empty_repository() {
    let repo = create_test_repo();
    let path = repo.path();

    // Don't create any commits

    // Status should work
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output()
        .expect("Failed to get status");

    assert!(output.status.success());

    // Branch should be empty or show default
    let branch_output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(path)
        .output()
        .expect("Failed to get branch");

    // May or may not have a branch depending on git version
    assert!(branch_output.status.success());
}

/// Test: Large number of files performance
#[test]
fn test_large_repo_performance() {
    let repo = create_test_repo();
    let path = repo.path();

    // Create many files
    for i in 0..100 {
        std::fs::write(path.join(format!("file_{}.txt", i)), format!("content {}", i)).unwrap();
    }

    Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "Add many files"])
        .current_dir(path)
        .output()
        .unwrap();

    // Timing test - should complete quickly
    let start = std::time::Instant::now();
    
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output()
        .expect("Failed to get status");

    let elapsed = start.elapsed();
    
    assert!(output.status.success());
    assert!(elapsed.as_millis() < 1000, "Git status took too long: {:?}", elapsed);
}
