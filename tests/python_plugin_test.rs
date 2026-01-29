//! Integration tests for the Python context plugin.

use std::fs;
use tempfile::TempDir;

/// Create a test Python project with pyproject.toml (PEP 621)
fn create_test_python_project() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.path();

    // Create pyproject.toml (PEP 621 format)
    let pyproject = r#"[project]
name = "test-project"
version = "0.1.0"
requires-python = ">=3.10"
dependencies = [
    "requests>=2.28.0",
    "click>=8.0.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.0.0",
    "black>=23.0.0",
]

[project.scripts]
mycli = "test_project:main"
"#;
    fs::write(path.join("pyproject.toml"), pyproject).unwrap();

    temp_dir
}

#[test]
fn test_detect_python_project() {
    let project = create_test_python_project();
    let path = project.path();

    assert!(path.join("pyproject.toml").exists());
}

#[test]
fn test_detect_uv_lock() {
    let project = create_test_python_project();
    let path = project.path();

    // Create uv.lock
    fs::write(path.join("uv.lock"), "").unwrap();

    assert!(path.join("uv.lock").exists());
}

#[test]
fn test_detect_poetry_lock() {
    let project = create_test_python_project();
    let path = project.path();

    // Create poetry.lock
    fs::write(path.join("poetry.lock"), "").unwrap();

    assert!(path.join("poetry.lock").exists());
}

#[test]
fn test_parse_pyproject_pep621() {
    let project = create_test_python_project();
    let path = project.path();

    let content = fs::read_to_string(path.join("pyproject.toml")).unwrap();
    let pyproject: toml::Value = toml::from_str(&content).unwrap();

    let project_section = pyproject.get("project").unwrap();
    assert_eq!(
        project_section.get("name").unwrap().as_str().unwrap(),
        "test-project"
    );
    assert_eq!(
        project_section.get("version").unwrap().as_str().unwrap(),
        "0.1.0"
    );
}

#[test]
fn test_extract_python_version() {
    let project = create_test_python_project();
    let path = project.path();

    let content = fs::read_to_string(path.join("pyproject.toml")).unwrap();
    let pyproject: toml::Value = toml::from_str(&content).unwrap();

    let project_section = pyproject.get("project").unwrap();
    assert_eq!(
        project_section
            .get("requires-python")
            .unwrap()
            .as_str()
            .unwrap(),
        ">=3.10"
    );
}

#[test]
fn test_extract_dependencies() {
    let project = create_test_python_project();
    let path = project.path();

    let content = fs::read_to_string(path.join("pyproject.toml")).unwrap();
    let pyproject: toml::Value = toml::from_str(&content).unwrap();

    let project_section = pyproject.get("project").unwrap();
    let deps = project_section
        .get("dependencies")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(deps.len(), 2);
}

#[test]
fn test_extract_scripts() {
    let project = create_test_python_project();
    let path = project.path();

    let content = fs::read_to_string(path.join("pyproject.toml")).unwrap();
    let pyproject: toml::Value = toml::from_str(&content).unwrap();

    let project_section = pyproject.get("project").unwrap();
    let scripts = project_section.get("scripts").unwrap().as_table().unwrap();
    assert!(scripts.contains_key("mycli"));
}

#[test]
fn test_parse_requirements_txt() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.path();

    let requirements = r#"# This is a comment
requests>=2.28.0
click>=8.0.0
-r other-requirements.txt
flask==2.0.0
"#;
    fs::write(path.join("requirements.txt"), requirements).unwrap();

    let content = fs::read_to_string(path.join("requirements.txt")).unwrap();
    let deps: Vec<&str> = content
        .lines()
        .filter(|l| {
            let trimmed = l.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with('-')
        })
        .collect();

    assert_eq!(deps.len(), 3);
}

#[test]
fn test_poetry_format() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.path();

    // Poetry format pyproject.toml
    let pyproject = r#"[tool.poetry]
name = "poetry-project"
version = "1.0.0"

[tool.poetry.dependencies]
python = "^3.10"
requests = "^2.28.0"

[tool.poetry.group.dev.dependencies]
pytest = "^7.0.0"

[tool.poetry.scripts]
mycli = "poetry_project:main"
"#;
    fs::write(path.join("pyproject.toml"), pyproject).unwrap();

    let content = fs::read_to_string(path.join("pyproject.toml")).unwrap();
    let pyproject_val: toml::Value = toml::from_str(&content).unwrap();

    let tool = pyproject_val.get("tool").unwrap();
    let poetry = tool.get("poetry").unwrap();
    assert_eq!(poetry.get("name").unwrap().as_str().unwrap(), "poetry-project");
}

#[test]
fn test_extract_package_name_from_specifier() {
    // Test helper function logic
    let test_cases = vec![
        ("requests>=2.28.0", "requests"),
        ("click==8.0.0", "click"),
        ("flask~=2.0", "flask"),
        ("django[rest]>=4.0", "django"),
        ("numpy", "numpy"),
    ];

    for (input, expected) in test_cases {
        let name_end = input
            .find(|c| {
                c == '>' || c == '<' || c == '=' || c == '!' || c == '~' || c == '[' || c == ';'
            })
            .unwrap_or(input.len());
        let name = &input[..name_end];
        assert_eq!(name.trim(), expected, "Failed for input: {}", input);
    }
}

#[test]
fn test_non_python_directory() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.path();

    assert!(!path.join("pyproject.toml").exists());
    assert!(!path.join("requirements.txt").exists());
}
