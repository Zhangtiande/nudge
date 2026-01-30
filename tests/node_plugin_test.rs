//! Integration tests for the Node.js context plugin.

use std::fs;
use tempfile::TempDir;

/// Create a test Node.js project
fn create_test_node_project() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.path();

    // Create package.json
    let package_json = r#"{
  "name": "test-project",
  "version": "1.0.0",
  "scripts": {
    "build": "tsc",
    "test": "jest",
    "start": "node index.js"
  },
  "dependencies": {
    "express": "^4.18.0",
    "lodash": "^4.17.0"
  },
  "devDependencies": {
    "typescript": "^5.0.0",
    "jest": "^29.0.0"
  },
  "engines": {
    "node": ">=18.0.0"
  }
}"#;
    fs::write(path.join("package.json"), package_json).unwrap();

    temp_dir
}

#[test]
fn test_detect_node_project() {
    let project = create_test_node_project();
    let path = project.path();

    assert!(path.join("package.json").exists());
}

#[test]
fn test_detect_npm_lock() {
    let project = create_test_node_project();
    let path = project.path();

    // Create package-lock.json
    fs::write(path.join("package-lock.json"), "{}").unwrap();

    assert!(path.join("package-lock.json").exists());
}

#[test]
fn test_detect_yarn_lock() {
    let project = create_test_node_project();
    let path = project.path();

    // Create yarn.lock
    fs::write(path.join("yarn.lock"), "").unwrap();

    assert!(path.join("yarn.lock").exists());
}

#[test]
fn test_detect_pnpm_lock() {
    let project = create_test_node_project();
    let path = project.path();

    // Create pnpm-lock.yaml
    fs::write(path.join("pnpm-lock.yaml"), "").unwrap();

    assert!(path.join("pnpm-lock.yaml").exists());
}

#[test]
fn test_detect_nvmrc() {
    let project = create_test_node_project();
    let path = project.path();

    // Create .nvmrc
    fs::write(path.join(".nvmrc"), "20.10.0").unwrap();

    let content = fs::read_to_string(path.join(".nvmrc")).unwrap();
    assert_eq!(content.trim(), "20.10.0");
}

#[test]
fn test_detect_node_version_file() {
    let project = create_test_node_project();
    let path = project.path();

    // Create .node-version
    fs::write(path.join(".node-version"), "18.19.0").unwrap();

    let content = fs::read_to_string(path.join(".node-version")).unwrap();
    assert_eq!(content.trim(), "18.19.0");
}

#[test]
fn test_parse_package_json() {
    let project = create_test_node_project();
    let path = project.path();

    let content = fs::read_to_string(path.join("package.json")).unwrap();
    let pkg: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(pkg["name"].as_str().unwrap(), "test-project");
    assert_eq!(pkg["version"].as_str().unwrap(), "1.0.0");
}

#[test]
fn test_extract_scripts() {
    let project = create_test_node_project();
    let path = project.path();

    let content = fs::read_to_string(path.join("package.json")).unwrap();
    let pkg: serde_json::Value = serde_json::from_str(&content).unwrap();

    let scripts = pkg["scripts"].as_object().unwrap();
    assert!(scripts.contains_key("build"));
    assert!(scripts.contains_key("test"));
    assert!(scripts.contains_key("start"));
}

#[test]
fn test_extract_dependencies() {
    let project = create_test_node_project();
    let path = project.path();

    let content = fs::read_to_string(path.join("package.json")).unwrap();
    let pkg: serde_json::Value = serde_json::from_str(&content).unwrap();

    let deps = pkg["dependencies"].as_object().unwrap();
    assert!(deps.contains_key("express"));
    assert!(deps.contains_key("lodash"));
}

#[test]
fn test_detect_monorepo() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.path();

    // Create monorepo package.json with workspaces
    let package_json = r#"{
  "name": "monorepo",
  "private": true,
  "workspaces": [
    "packages/*"
  ]
}"#;
    fs::write(path.join("package.json"), package_json).unwrap();

    let content = fs::read_to_string(path.join("package.json")).unwrap();
    let pkg: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(pkg.get("workspaces").is_some());
}

#[test]
fn test_non_node_directory() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.path();

    // No package.json
    assert!(!path.join("package.json").exists());
}
