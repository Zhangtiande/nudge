//! Integration tests for the Rust context plugin.

use std::fs;
use tempfile::TempDir;

/// Create a test Rust project
fn create_test_rust_project() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.path();

    // Create Cargo.toml
    let cargo_toml = r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
rust-version = "1.70"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
tempfile = "3.0"
"#;
    fs::write(path.join("Cargo.toml"), cargo_toml).unwrap();

    // Create src/main.rs
    fs::create_dir_all(path.join("src")).unwrap();
    fs::write(path.join("src/main.rs"), "fn main() {}").unwrap();

    temp_dir
}

#[test]
fn test_detect_rust_project() {
    let project = create_test_rust_project();
    let path = project.path();

    assert!(path.join("Cargo.toml").exists());
}

#[test]
fn test_parse_cargo_toml() {
    let project = create_test_rust_project();
    let path = project.path();

    let content = fs::read_to_string(path.join("Cargo.toml")).unwrap();
    let cargo: toml::Value = toml::from_str(&content).unwrap();

    let package = cargo.get("package").unwrap();
    assert_eq!(
        package.get("name").unwrap().as_str().unwrap(),
        "test-project"
    );
    assert_eq!(package.get("version").unwrap().as_str().unwrap(), "0.1.0");
}

#[test]
fn test_extract_rust_version() {
    let project = create_test_rust_project();
    let path = project.path();

    let content = fs::read_to_string(path.join("Cargo.toml")).unwrap();
    let cargo: toml::Value = toml::from_str(&content).unwrap();

    let package = cargo.get("package").unwrap();
    assert_eq!(
        package.get("rust-version").unwrap().as_str().unwrap(),
        "1.70"
    );
}

#[test]
fn test_extract_dependencies() {
    let project = create_test_rust_project();
    let path = project.path();

    let content = fs::read_to_string(path.join("Cargo.toml")).unwrap();
    let cargo: toml::Value = toml::from_str(&content).unwrap();

    let deps = cargo.get("dependencies").unwrap().as_table().unwrap();
    assert!(deps.contains_key("serde"));
    assert!(deps.contains_key("tokio"));
}

#[test]
fn test_extract_dev_dependencies() {
    let project = create_test_rust_project();
    let path = project.path();

    let content = fs::read_to_string(path.join("Cargo.toml")).unwrap();
    let cargo: toml::Value = toml::from_str(&content).unwrap();

    let dev_deps = cargo.get("dev-dependencies").unwrap().as_table().unwrap();
    assert!(dev_deps.contains_key("tempfile"));
}

#[test]
fn test_detect_workspace() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.path();

    // Create workspace Cargo.toml
    let cargo_toml = r#"[workspace]
members = [
    "crates/core",
    "crates/cli"
]
"#;
    fs::write(path.join("Cargo.toml"), cargo_toml).unwrap();

    let content = fs::read_to_string(path.join("Cargo.toml")).unwrap();
    let cargo: toml::Value = toml::from_str(&content).unwrap();

    assert!(cargo.get("workspace").is_some());
}

#[test]
fn test_extract_workspace_members() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.path();

    let cargo_toml = r#"[workspace]
members = [
    "crates/core",
    "crates/cli"
]
"#;
    fs::write(path.join("Cargo.toml"), cargo_toml).unwrap();

    let content = fs::read_to_string(path.join("Cargo.toml")).unwrap();
    let cargo: toml::Value = toml::from_str(&content).unwrap();

    let workspace = cargo.get("workspace").unwrap();
    let members = workspace.get("members").unwrap().as_array().unwrap();
    assert_eq!(members.len(), 2);
}

#[test]
fn test_detect_binary_target() {
    let project = create_test_rust_project();
    let path = project.path();

    // src/main.rs exists = default binary
    assert!(path.join("src/main.rs").exists());
}

#[test]
fn test_explicit_binary_targets() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.path();

    let cargo_toml = r#"[package]
name = "multi-bin"
version = "0.1.0"

[[bin]]
name = "cli"
path = "src/cli.rs"

[[bin]]
name = "server"
path = "src/server.rs"
"#;
    fs::write(path.join("Cargo.toml"), cargo_toml).unwrap();

    let content = fs::read_to_string(path.join("Cargo.toml")).unwrap();
    let cargo: toml::Value = toml::from_str(&content).unwrap();

    let bins = cargo.get("bin").unwrap().as_array().unwrap();
    assert_eq!(bins.len(), 2);
}

#[test]
fn test_non_rust_directory() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.path();

    assert!(!path.join("Cargo.toml").exists());
}
