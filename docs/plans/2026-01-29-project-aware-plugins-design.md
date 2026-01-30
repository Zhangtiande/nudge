# Project-Aware Plugins Design

Date: 2026-01-29

## Overview

This document outlines the design for three new project-aware context plugins (Node.js, Rust, Python) and the refactoring of installation scripts to eliminate inline configurations.

## Goals

1. Implement Node.js, Rust, and Python (uv) plugins for project context awareness
2. Refactor installation scripts to use template files exclusively
3. Update ROADMAP.md to reflect progress

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Data collection | File parsing only | No external commands; reliable within 100ms timeout |
| Timeout | 100ms | Matches Docker plugin; sufficient for file I/O |
| Priority | 45 | Same as Docker; lower than Git (50) |
| Activation | Hybrid (command prefix OR feature file) | Consistent with existing plugins |
| Package manager detection | Lock file based | Deterministic; no command execution needed |

## Data Structures

### Node.js Plugin

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeContext {
    pub package_manager: PackageManager,
    pub name: Option<String>,
    pub version: Option<String>,
    pub node_version: Option<String>,      // from .nvmrc / .node-version / engines
    pub scripts: Vec<String>,
    pub dependencies: Vec<String>,
    pub dev_dependencies: Vec<String>,
    pub is_monorepo: bool,                 // workspaces field present
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageManager {
    Npm,      // package-lock.json
    Yarn,     // yarn.lock
    Pnpm,     // pnpm-lock.yaml
    Unknown,
}
```

### Rust Plugin

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustContext {
    pub name: Option<String>,
    pub version: Option<String>,
    pub rust_version: Option<String>,      // rust-version field
    pub is_workspace: bool,
    pub workspace_members: Vec<String>,
    pub dependencies: Vec<String>,
    pub dev_dependencies: Vec<String>,
    pub binaries: Vec<String>,             // [[bin]] targets
}
```

### Python Plugin (uv-first)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonContext {
    pub package_manager: PythonPackageManager,
    pub name: Option<String>,
    pub version: Option<String>,
    pub python_version: Option<String>,    // requires-python
    pub dependencies: Vec<String>,
    pub dev_dependencies: Vec<String>,
    pub scripts: Vec<String>,              // entry points
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PythonPackageManager {
    Uv,       // uv.lock
    Poetry,   // poetry.lock (reserved for future)
    Pip,      // requirements.txt only
    Unknown,
}
```

## Activation Strategies

| Plugin | Feature Files | Command Prefixes |
|--------|---------------|------------------|
| Node.js | `package.json` | `npm`, `yarn`, `pnpm`, `node`, `npx` |
| Rust | `Cargo.toml` | `cargo`, `rustc` |
| Python | `pyproject.toml`, `uv.lock` | `uv`, `python`, `pip` |

## Configuration

```rust
// Added to src/config.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePluginConfig {
    pub enabled: bool,           // default: true
    pub timeout_ms: u64,         // default: 100
    pub priority: Option<u8>,    // default: 45
    pub max_dependencies: usize, // default: 50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustPluginConfig {
    pub enabled: bool,           // default: true
    pub timeout_ms: u64,         // default: 100
    pub priority: Option<u8>,    // default: 45
    pub max_dependencies: usize, // default: 50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonPluginConfig {
    pub enabled: bool,           // default: true
    pub timeout_ms: u64,         // default: 100
    pub priority: Option<u8>,    // default: 45
    pub max_dependencies: usize, // default: 50
}

// PluginsConfig extended
pub struct PluginsConfig {
    pub git: GitPluginConfig,
    pub docker: DockerPluginConfig,
    pub node: NodePluginConfig,      // new
    pub rust: RustPluginConfig,      // new
    pub python: PythonPluginConfig,  // new
    pub plugin_dir: Option<PathBuf>,
}
```

## File Changes

### New Files

| File | Description |
|------|-------------|
| `src/daemon/plugins/builtin/node.rs` | Node.js plugin implementation |
| `src/daemon/plugins/builtin/rust.rs` | Rust plugin implementation |
| `src/daemon/plugins/builtin/python.rs` | Python plugin implementation |
| `tests/node_plugin_test.rs` | Node.js plugin tests |
| `tests/rust_plugin_test.rs` | Rust plugin tests |
| `tests/python_plugin_test.rs` | Python plugin tests |

### Modified Files

| File | Changes |
|------|---------|
| `src/config.rs` | Add three plugin config structs |
| `src/daemon/plugins/builtin/mod.rs` | Export new plugins |
| `src/daemon/context/mod.rs` | Register plugins in `create_plugin_manager()` |
| `config/config.default.yaml.template` | Add plugin default configs |
| `config/config.user.yaml.template` | Add plugin config examples |

### Script Refactoring

**Constraint**: Installation scripts must NOT contain inline configurations. All configurations must come from template files.

| Script | Changes |
|--------|---------|
| `scripts/install.ps1` | Remove inline config; copy from template |
| `scripts/install.sh` | Remove inline config; copy from template |
| `shell/setup-shell.ps1` | Remove inline config; use template |
| `shell/setup-shell.sh` | Remove inline config; use template |

## Implementation Flow

Each plugin follows the same pattern:

```rust
impl XxxPlugin {
    async fn collect(&self, cwd: &Path) -> Result<PluginContextData> {
        // 1. Read config file (package.json / Cargo.toml / pyproject.toml)
        // 2. Detect package manager via lock file
        // 3. Extract version requirements
        // 4. Build context struct
        // 5. Return PluginContextData
    }
}
```

## Plugin Registration

```rust
// In src/daemon/context/mod.rs

fn create_plugin_manager(config: &Config) -> plugin::PluginManager {
    PluginManager::new()
        // ... existing git and docker plugins ...
        .register(
            Box::new(NodePlugin::new(config.plugins.node.clone())),
            Box::new(CombinedActivation::new(vec![
                Box::new(FeatureFileActivation::new(vec!["package.json"])),
                Box::new(CommandPrefixActivation::new(vec![
                    "npm", "yarn", "pnpm", "node", "npx"
                ])),
            ])),
            config.plugins.node.enabled,
            config.plugins.node.timeout_ms,
            config.plugins.node.priority.unwrap_or(45),
        )
        .register(
            Box::new(RustPlugin::new(config.plugins.rust.clone())),
            Box::new(CombinedActivation::new(vec![
                Box::new(FeatureFileActivation::new(vec!["Cargo.toml"])),
                Box::new(CommandPrefixActivation::new(vec!["cargo", "rustc"])),
            ])),
            config.plugins.rust.enabled,
            config.plugins.rust.timeout_ms,
            config.plugins.rust.priority.unwrap_or(45),
        )
        .register(
            Box::new(PythonPlugin::new(config.plugins.python.clone())),
            Box::new(CombinedActivation::new(vec![
                Box::new(FeatureFileActivation::new(vec![
                    "pyproject.toml", "uv.lock"
                ])),
                Box::new(CommandPrefixActivation::new(vec![
                    "uv", "python", "pip"
                ])),
            ])),
            config.plugins.python.enabled,
            config.plugins.python.timeout_ms,
            config.plugins.python.priority.unwrap_or(45),
        )
}
```

## Test Strategy

Each plugin test file should cover:

1. Feature file detection (`is_applicable`)
2. Config file parsing (valid JSON/TOML)
3. Package manager detection (lock file based)
4. Dependency extraction with `max_dependencies` limit
5. Graceful handling of malformed files
6. Timeout behavior

## ROADMAP Update

Update `ROADMAP.md` to mark Node.js, Rust, Python plugins as completed in v0.4.0.
