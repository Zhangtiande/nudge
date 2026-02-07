use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use tracing::{debug, info, warn};

use crate::paths::AppPaths;

const CONFIG_ENV: &str = "NUDGE_CONFIG";
const LEGACY_CONFIG_ENV: &str = "SMARTSHELL_CONFIG";

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub model: ModelConfig,
    pub context: ContextConfig,
    pub plugins: PluginsConfig,
    pub trigger: TriggerConfig,
    pub cache: CacheConfig,
    pub privacy: PrivacyConfig,
    pub log: LogConfig,
    pub diagnosis: DiagnosisConfig,
    pub system_prompt: Option<String>,
}

/// Model/LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelConfig {
    /// API endpoint URL
    pub endpoint: String,
    /// Model name/identifier
    pub model_name: String,
    /// API key (directly configured, takes precedence over api_key_env)
    pub api_key: Option<String>,
    /// Environment variable containing API key (fallback if api_key is not set)
    pub api_key_env: Option<String>,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434/v1".to_string(),
            model_name: "codellama:7b".to_string(),
            api_key: None,
            api_key_env: None,
            timeout_ms: 5000,
        }
    }
}

/// Context collection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ContextConfig {
    /// Number of history commands to include
    pub history_window: usize,
    /// Include CWD file listing
    pub include_cwd_listing: bool,
    /// Include last exit code
    pub include_exit_code: bool,
    /// Include system information (OS, architecture, shell type, etc.)
    pub include_system_info: bool,
    /// Enable similar commands search from history
    pub similar_commands_enabled: bool,
    /// Number of history commands to search for similar commands
    pub similar_commands_window: usize,
    /// Maximum number of similar commands to return
    pub similar_commands_max: usize,
    /// Max files to include in CWD listing
    pub max_files_in_listing: usize,
    /// Max total context tokens
    pub max_total_tokens: usize,
    /// Priority levels for truncation
    pub priorities: PriorityConfig,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            history_window: 20,
            include_cwd_listing: true,
            include_exit_code: true,
            include_system_info: true,
            similar_commands_enabled: true,
            similar_commands_window: 200,
            similar_commands_max: 5,
            max_files_in_listing: 50,
            max_total_tokens: 4000,
            priorities: PriorityConfig::default(),
        }
    }
}

/// Priority levels for context sources (higher = keep when truncating)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PriorityConfig {
    pub history: u8,
    pub cwd_listing: u8,
    pub plugins: u8,
}

impl Default for PriorityConfig {
    fn default() -> Self {
        Self {
            history: 80,
            cwd_listing: 60,
            plugins: 40,
        }
    }
}

/// Plugin settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PluginsConfig {
    pub git: GitPluginConfig,
    pub docker: DockerPluginConfig,
    pub node: NodePluginConfig,
    pub rust: RustPluginConfig,
    pub python: PythonPluginConfig,
}

/// Git plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GitPluginConfig {
    pub enabled: bool,
    pub depth: GitDepth,
    pub recent_commits: usize,
    pub priority: Option<u8>,
}

impl Default for GitPluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            depth: GitDepth::Standard,
            recent_commits: 5,
            priority: Some(50),
        }
    }
}

/// Git context depth level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitDepth {
    Light,
    #[default]
    Standard,
    Detailed,
}

/// Docker plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DockerPluginConfig {
    pub enabled: bool,
    pub timeout_ms: u64,
    pub priority: Option<u8>,
    pub max_containers: usize,
    pub max_images: usize,
    pub show_containers: bool,
    pub include_compose: bool,
    pub include_dockerfile: bool,
}

impl Default for DockerPluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_ms: 100,
            priority: Some(45),
            max_containers: 10,
            max_images: 10,
            show_containers: true,
            include_compose: true,
            include_dockerfile: true,
        }
    }
}

/// Node.js plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NodePluginConfig {
    pub enabled: bool,
    pub timeout_ms: u64,
    pub priority: Option<u8>,
    pub max_dependencies: usize,
}

impl Default for NodePluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_ms: 100,
            priority: Some(45),
            max_dependencies: 50,
        }
    }
}

/// Rust plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RustPluginConfig {
    pub enabled: bool,
    pub timeout_ms: u64,
    pub priority: Option<u8>,
    pub max_dependencies: usize,
}

impl Default for RustPluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_ms: 100,
            priority: Some(45),
            max_dependencies: 50,
        }
    }
}

/// Python plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PythonPluginConfig {
    pub enabled: bool,
    pub timeout_ms: u64,
    pub priority: Option<u8>,
    pub max_dependencies: usize,
}

impl Default for PythonPluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_ms: 100,
            priority: Some(45),
            max_dependencies: 50,
        }
    }
}

/// Trigger behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TriggerConfig {
    pub mode: TriggerMode,
    pub hotkey: String,
    pub auto_delay_ms: u64,
    pub zsh_ghost_owner: ZshGhostOwner,
    pub zsh_overlay_backend: ZshOverlayBackend,
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            mode: TriggerMode::Manual,
            hotkey: r"\C-e".to_string(),
            auto_delay_ms: 500,
            zsh_ghost_owner: ZshGhostOwner::Auto,
            zsh_overlay_backend: ZshOverlayBackend::Message,
        }
    }
}

/// Cache settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    pub capacity: usize,
    pub prefix_bytes: usize,
    pub ttl_auto_ms: u64,
    pub ttl_manual_ms: u64,
    pub ttl_negative_ms: u64,
    pub stale_ratio: f32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            capacity: 1024,
            prefix_bytes: 80,
            ttl_auto_ms: 300000,    // 5 minutes
            ttl_manual_ms: 600000,  // 10 minutes
            ttl_negative_ms: 30000, // 30 seconds
            stale_ratio: 0.8,
        }
    }
}

/// Trigger mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerMode {
    #[default]
    Manual,
    Auto,
}

/// Zsh ghost text owner selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ZshGhostOwner {
    /// Auto-detect: prefer zsh-autosuggestions when available
    #[default]
    Auto,
    /// Use nudge as ghost text owner
    Nudge,
    /// Use zsh-autosuggestions as ghost text owner
    Autosuggestions,
}

/// Zsh overlay rendering backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ZshOverlayBackend {
    /// Render using `zle -M` message line
    #[default]
    Message,
    /// Render in right prompt (RPS1)
    Rprompt,
}

/// Privacy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PrivacyConfig {
    pub sanitize_enabled: bool,
    pub custom_patterns: Vec<String>,
    pub block_dangerous: bool,
    pub custom_blocked: Vec<String>,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            sanitize_enabled: true,
            custom_patterns: Vec::new(),
            block_dangerous: true,
            custom_blocked: Vec::new(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LogConfig {
    /// Log level: trace, debug, info, warn, error
    pub level: String,
    /// Enable file logging
    pub file_enabled: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_enabled: false,
        }
    }
}

/// Error diagnosis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DiagnosisConfig {
    /// Enable/disable error diagnosis feature
    pub enabled: bool,
    /// Zsh: capture stderr to file during command execution
    pub capture_stderr: bool,
    /// Zsh: show suggested fix as gray inline text
    pub auto_suggest: bool,
    /// Maximum stderr size to send to LLM (bytes)
    pub max_stderr_size: usize,
    /// Timeout for diagnosis request (ms)
    pub timeout_ms: u64,
    /// Commands that should skip stderr capture (interactive programs)
    /// These programs need real-time stderr output (e.g., vim, ssh, top)
    pub interactive_commands: Vec<String>,
}

impl Default for DiagnosisConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            capture_stderr: true,
            auto_suggest: true,
            max_stderr_size: 4096,
            timeout_ms: 5000,
            interactive_commands: vec![
                // Editors
                "vim".to_string(),
                "nvim".to_string(),
                "vi".to_string(),
                "nano".to_string(),
                "emacs".to_string(),
                "code".to_string(),
                // Remote access
                "ssh".to_string(),
                "telnet".to_string(),
                "mosh".to_string(),
                // Interactive tools
                "top".to_string(),
                "htop".to_string(),
                "btop".to_string(),
                "less".to_string(),
                "more".to_string(),
                "man".to_string(),
                // Fuzzy finders
                "fzf".to_string(),
                "sk".to_string(),
                // Terminal multiplexers
                "tmux".to_string(),
                "screen".to_string(),
                // Interactive shells
                "python".to_string(),
                "python3".to_string(),
                "ipython".to_string(),
                "node".to_string(),
                "irb".to_string(),
                "psql".to_string(),
                "mysql".to_string(),
                "sqlite3".to_string(),
                // Other interactive programs
                "watch".to_string(),
                "tail".to_string(),
            ],
        }
    }
}

impl Config {
    /// Load configuration with layered approach:
    /// 1. Start with built-in defaults
    /// 2. Override with config.default.yaml (ships with app, updated on upgrade)
    /// 3. Override with config.yaml (user customizations, preserved on upgrade)
    pub fn load() -> Result<Self> {
        // Check for environment variable override (skips layered loading)
        if let Some((env_name, config_path)) = Self::resolve_override_config_path() {
            info!(
                "Loading config from {}: {}",
                env_name,
                config_path.display()
            );
            return Self::load_from_path(&config_path);
        }

        // Start with built-in defaults as YAML value
        let mut merged_value: Value =
            serde_yaml::to_value(Self::default()).context("Failed to serialize default config")?;

        // Layer 1: Load config.default.yaml if present (ships with app)
        let base_path = Self::base_config_path();
        if let Some(base_value) = Self::load_yaml_layer(&base_path, "base")? {
            merged_value = Self::deep_merge(merged_value, base_value);
            debug!("Merged base config: {}", base_path.display());
        }

        // Layer 2: Load config.yaml if present (user customizations)
        let user_path = Self::default_config_path();
        if let Some(user_value) = Self::load_yaml_layer(&user_path, "user")? {
            merged_value = Self::deep_merge(merged_value, user_value);
            debug!("Merged user config: {}", user_path.display());
        }

        // Deserialize merged config
        let config: Self =
            serde_yaml::from_value(merged_value).context("Failed to deserialize merged config")?;

        config.validate()?;
        Self::log_loaded_summary(&config);

        Ok(config)
    }

    /// Resolve config override path from environment variables.
    /// Priority: NUDGE_CONFIG > SMARTSHELL_CONFIG (legacy fallback)
    fn resolve_override_config_path() -> Option<(&'static str, PathBuf)> {
        let nudge_override = std::env::var_os(CONFIG_ENV).map(PathBuf::from);
        let legacy_override = std::env::var_os(LEGACY_CONFIG_ENV).map(PathBuf::from);

        match (nudge_override, legacy_override) {
            (Some(path), Some(_)) => {
                warn!(
                    "Both {} and {} are set. Using {}.",
                    CONFIG_ENV, LEGACY_CONFIG_ENV, CONFIG_ENV
                );
                Some((CONFIG_ENV, path))
            }
            (Some(path), None) => Some((CONFIG_ENV, path)),
            (None, Some(path)) => {
                warn!(
                    "{} is deprecated. Please migrate to {}.",
                    LEGACY_CONFIG_ENV, CONFIG_ENV
                );
                Some((LEGACY_CONFIG_ENV, path))
            }
            (None, None) => None,
        }
    }

    /// Load and parse an optional YAML layer.
    /// Returns Ok(None) when file does not exist or is intentionally empty.
    fn load_yaml_layer(path: &Path, layer_name: &str) -> Result<Option<Value>> {
        let contents = match std::fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => {
                return Err(e).with_context(|| {
                    format!(
                        "Failed to read {} config file: {}",
                        layer_name,
                        path.display()
                    )
                });
            }
        };

        if Self::is_empty_yaml_document(&contents) {
            debug!("Skipped empty {} config: {}", layer_name, path.display());
            return Ok(None);
        }

        let value = serde_yaml::from_str::<Value>(&contents).with_context(|| {
            format!(
                "Failed to parse {} config file: {}",
                layer_name,
                path.display()
            )
        })?;

        Ok(Some(value))
    }

    fn is_empty_yaml_document(contents: &str) -> bool {
        let trimmed = contents.trim();
        trimmed.is_empty() || trimmed == "---"
    }

    /// Deep merge two YAML values. Values from `override_value` take precedence.
    /// For mappings, keys are merged recursively.
    /// For other types, override completely replaces base.
    fn deep_merge(base: Value, override_value: Value) -> Value {
        match (base, override_value) {
            (Value::Mapping(mut base_map), Value::Mapping(override_map)) => {
                for (key, override_val) in override_map {
                    let merged_val = if let Some(base_val) = base_map.remove(&key) {
                        Self::deep_merge(base_val, override_val)
                    } else {
                        override_val
                    };
                    base_map.insert(key, merged_val);
                }
                Value::Mapping(base_map)
            }
            // For non-mapping types, override wins
            (_, override_value) => override_value,
        }
    }

    /// Load configuration from a specific path (no layering, direct load)
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        debug!(
            "Config file loaded ({} bytes): {}",
            contents.len(),
            path.display()
        );

        let config: Self = serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        config.validate()?;
        Self::log_loaded_summary(&config);

        Ok(config)
    }

    fn log_loaded_summary(config: &Self) {
        // Log loaded configuration (concise single-line summary)
        info!(
            "Config loaded: model={} endpoint={} timeout={}ms",
            config.model.model_name, config.model.endpoint, config.model.timeout_ms
        );
        debug!(
            "Config details: history_window={} git_enabled={} system_info={} cwd_listing={}",
            config.context.history_window,
            config.plugins.git.enabled,
            config.context.include_system_info,
            config.context.include_cwd_listing
        );
    }

    /// Get the base config file path (config.default.yaml - ships with app)
    pub fn base_config_path() -> PathBuf {
        AppPaths::default_config_path()
    }

    /// Get the user config file path (config.yaml - user customizations)
    pub fn default_config_path() -> PathBuf {
        AppPaths::user_config_path()
    }

    /// Get the socket path for IPC
    /// On Unix: ~/.nudge/run/nudge.sock (Unix Domain Socket)
    /// On Windows: \\.\pipe\nudge_{username} (Named Pipe)
    #[cfg(unix)]
    pub fn socket_path() -> PathBuf {
        AppPaths::socket_path()
    }

    /// Get the socket path for IPC (Windows Named Pipe)
    #[cfg(windows)]
    pub fn socket_path() -> PathBuf {
        AppPaths::socket_path()
    }

    /// Get the PID file path
    pub fn pid_path() -> PathBuf {
        AppPaths::pid_path()
    }

    /// Get the log directory path
    pub fn log_dir() -> PathBuf {
        AppPaths::logs_dir()
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        if self.model.timeout_ms == 0 {
            anyhow::bail!("model.timeout_ms must be greater than 0");
        }

        if self.context.history_window == 0 {
            anyhow::bail!("context.history_window must be greater than 0");
        }

        if self.context.max_total_tokens == 0 {
            anyhow::bail!("context.max_total_tokens must be greater than 0");
        }

        if self.trigger.auto_delay_ms == 0 {
            anyhow::bail!("trigger.auto_delay_ms must be greater than 0");
        }

        if self.cache.capacity == 0 {
            anyhow::bail!("cache.capacity must be greater than 0");
        }

        if self.cache.prefix_bytes == 0 {
            anyhow::bail!("cache.prefix_bytes must be greater than 0");
        }

        if self.cache.ttl_auto_ms == 0 {
            anyhow::bail!("cache.ttl_auto_ms must be greater than 0");
        }

        if self.cache.ttl_manual_ms == 0 {
            anyhow::bail!("cache.ttl_manual_ms must be greater than 0");
        }

        if self.cache.ttl_negative_ms == 0 {
            anyhow::bail!("cache.ttl_negative_ms must be greater than 0");
        }

        if !self.cache.stale_ratio.is_finite() || !(0.0..=1.0).contains(&self.cache.stale_ratio) {
            anyhow::bail!("cache.stale_ratio must be between 0.0 and 1.0");
        }

        if self.diagnosis.max_stderr_size == 0 {
            anyhow::bail!("diagnosis.max_stderr_size must be greater than 0");
        }

        if self.diagnosis.timeout_ms == 0 {
            anyhow::bail!("diagnosis.timeout_ms must be greater than 0");
        }

        Self::validate_priority(
            "context.priorities.history",
            self.context.priorities.history,
        )?;
        Self::validate_priority(
            "context.priorities.cwd_listing",
            self.context.priorities.cwd_listing,
        )?;
        Self::validate_priority(
            "context.priorities.plugins",
            self.context.priorities.plugins,
        )?;
        if let Some(priority) = self.plugins.git.priority {
            Self::validate_priority("plugins.git.priority", priority)?;
        }
        if let Some(priority) = self.plugins.docker.priority {
            Self::validate_priority("plugins.docker.priority", priority)?;
        }
        if let Some(priority) = self.plugins.node.priority {
            Self::validate_priority("plugins.node.priority", priority)?;
        }
        if let Some(priority) = self.plugins.rust.priority {
            Self::validate_priority("plugins.rust.priority", priority)?;
        }
        if let Some(priority) = self.plugins.python.priority {
            Self::validate_priority("plugins.python.priority", priority)?;
        }

        Ok(())
    }

    fn validate_priority(field: &str, priority: u8) -> Result<()> {
        if !(1..=100).contains(&priority) {
            anyhow::bail!("{} must be between 1 and 100", field);
        }
        Ok(())
    }

    /// Check if LLM configuration is properly set up
    /// Returns Ok(()) if valid, Err with helpful message if not
    pub fn validate_llm_config(&self) -> Result<()> {
        // Check endpoint
        if self.model.endpoint.is_empty() {
            anyhow::bail!(
                "LLM endpoint is not configured. Please set 'model.endpoint' in your config file."
            );
        }

        // Check model name
        if self.model.model_name.is_empty() {
            anyhow::bail!("LLM model name is not configured. Please set 'model.model_name' in your config file.");
        }

        // For non-local endpoints, check if API key is configured
        let is_local = self.model.endpoint.contains("localhost")
            || self.model.endpoint.contains("127.0.0.1")
            || self.model.endpoint.contains("0.0.0.0");

        if !is_local {
            // Check if api_key is set directly
            let has_direct_key = self.model.api_key.as_ref().is_some_and(|k| !k.is_empty());

            // Check if api_key_env is set and the env var exists
            let has_env_key = self
                .model
                .api_key_env
                .as_ref()
                .is_some_and(|env_var| !env_var.is_empty() && std::env::var(env_var).is_ok());

            if !has_direct_key && !has_env_key {
                let config_path = Self::default_config_path().display().to_string();

                let mut msg = format!(
                    "API key is required for remote LLM endpoint '{}'\n\n",
                    self.model.endpoint
                );
                msg.push_str("Please configure one of the following in your config file:\n\n");
                msg.push_str("  Option 1 - Direct API key:\n");
                msg.push_str("    model:\n");
                msg.push_str("      api_key: \"your-api-key-here\"\n\n");
                msg.push_str("  Option 2 - Environment variable (recommended for security):\n");
                msg.push_str("    model:\n");
                msg.push_str("      api_key_env: \"OPENAI_API_KEY\"\n\n");
                msg.push_str(&format!("Config file location: {}", config_path));

                anyhow::bail!(msg);
            }
        }

        Ok(())
    }

    /// Get a user-friendly summary of LLM configuration status
    pub fn llm_config_summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str(&format!("  Endpoint: {}\n", self.model.endpoint));
        summary.push_str(&format!("  Model: {}\n", self.model.model_name));

        let auth_status = if self.model.api_key.as_ref().is_some_and(|k| !k.is_empty()) {
            "Configured (direct)"
        } else if let Some(env_var) = &self.model.api_key_env {
            if std::env::var(env_var).is_ok() {
                "Configured (via env)"
            } else {
                "NOT SET (env var missing)"
            }
        } else {
            "Not required (local)"
        };
        summary.push_str(&format!("  API Key: {}", auth_status));

        summary
    }
}

/// Platform detection and OS-specific logic
/// Foundation module for Phase 1 - methods marked #[allow(dead_code)] until integrated in Phase 2
#[derive(Debug)]
#[allow(dead_code)]
pub struct Platform {
    pub os: OsType,
    pub shell: ShellType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OsType {
    Linux,
    MacOS,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    Bash,
    Zsh,
    PowerShell,
    Cmd,
    Unknown,
}

impl Platform {
    /// Detect current platform at runtime
    #[allow(dead_code)]
    pub fn detect() -> Result<Self> {
        let os = if cfg!(target_os = "macos") {
            OsType::MacOS
        } else if cfg!(target_os = "linux") {
            OsType::Linux
        } else if cfg!(target_os = "windows") {
            OsType::Windows
        } else {
            anyhow::bail!("Unsupported operating system: {}", std::env::consts::OS);
        };

        let shell = Self::detect_shell();

        Ok(Self { os, shell })
    }

    /// Detect current shell from environment
    #[allow(dead_code)]
    fn detect_shell() -> ShellType {
        // Check SHELL environment variable (Unix)
        if let Ok(shell_path) = std::env::var("SHELL") {
            if shell_path.contains("bash") {
                return ShellType::Bash;
            } else if shell_path.contains("zsh") {
                return ShellType::Zsh;
            }
        }

        // Check PSModulePath (PowerShell)
        if std::env::var("PSModulePath").is_ok() {
            return ShellType::PowerShell;
        }

        // Check COMSPEC (CMD)
        if let Ok(comspec) = std::env::var("COMSPEC") {
            if comspec.to_lowercase().contains("cmd.exe") {
                return ShellType::Cmd;
            }
        }

        ShellType::Unknown
    }

    /// Get platform-specific nudge root directory
    #[allow(dead_code)]
    pub fn config_dir(&self) -> Result<PathBuf> {
        Ok(AppPaths::root_dir())
    }

    /// Get shell integration script path for current shell
    #[allow(dead_code)]
    pub fn integration_script_path(&self) -> Result<PathBuf> {
        let filename = match self.shell {
            ShellType::Bash => "integration.bash",
            ShellType::Zsh => "integration.zsh",
            ShellType::PowerShell => "integration.ps1",
            ShellType::Cmd => "integration.cmd",
            ShellType::Unknown => "integration.bash", // fallback
        };
        Ok(AppPaths::shell_dir().join(filename))
    }

    /// Get shell profile path (for setup command)
    #[allow(dead_code)]
    pub fn shell_profile_path(&self) -> Result<PathBuf> {
        match self.shell {
            ShellType::Bash => {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join(".bashrc"))
            }
            ShellType::Zsh => {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join(".zshrc"))
            }
            ShellType::PowerShell => {
                // Try to get actual $PROFILE path from PowerShell
                #[cfg(windows)]
                {
                    use std::process::Command;

                    // Try PowerShell 7 first (pwsh), then fall back to Windows PowerShell
                    for shell in ["pwsh", "powershell"] {
                        if let Ok(output) = Command::new(shell)
                            .args([
                                "-NoProfile",
                                "-NonInteractive",
                                "-Command",
                                "Write-Output $PROFILE",
                            ])
                            .output()
                        {
                            if output.status.success() {
                                let profile_path =
                                    String::from_utf8_lossy(&output.stdout).trim().to_string();
                                if !profile_path.is_empty() {
                                    return Ok(PathBuf::from(profile_path));
                                }
                            }
                        }
                    }
                }

                // Fallback: Check PROFILE env var
                if let Ok(profile) = std::env::var("PROFILE") {
                    return Ok(PathBuf::from(profile));
                }

                // Last resort: Default location
                let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME"))?;
                Ok(PathBuf::from(home)
                    .join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1"))
            }
            ShellType::Cmd => {
                anyhow::bail!("CMD does not support profile-based integration")
            }
            ShellType::Unknown => {
                anyhow::bail!("Cannot determine shell profile path for unknown shell")
            }
        }
    }

    /// Get the path to the dynamic library for FFI mode
    ///
    /// Returns the platform-specific library path:
    /// - macOS: `~/.nudge/lib/libnudge.dylib`
    /// - Linux: `~/.nudge/lib/libnudge.so`
    /// - Windows: None (FFI not supported on Windows)
    #[allow(dead_code)]
    pub fn lib_path(&self) -> Option<PathBuf> {
        match self.os {
            OsType::MacOS => Some(AppPaths::lib_dir().join("libnudge.dylib")),
            OsType::Linux => Some(AppPaths::lib_dir().join("libnudge.so")),
            OsType::Windows => None, // FFI not supported on Windows
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.os, self.shell)
    }
}

impl std::fmt::Display for OsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OsType::MacOS => write!(f, "macOS"),
            OsType::Linux => write!(f, "Linux"),
            OsType::Windows => write!(f, "Windows"),
        }
    }
}

impl std::fmt::Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellType::Bash => write!(f, "bash"),
            ShellType::Zsh => write!(f, "zsh"),
            ShellType::PowerShell => write!(f, "powershell"),
            ShellType::Cmd => write!(f, "cmd"),
            ShellType::Unknown => write!(f, "unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    use serde_yaml::Value;
    use tempfile::NamedTempFile;

    use super::{Config, ZshGhostOwner, ZshOverlayBackend, CONFIG_ENV, LEGACY_CONFIG_ENV};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvVarGuard {
        originals: Vec<(String, Option<OsString>)>,
    }

    impl EnvVarGuard {
        fn set(vars: &[(&str, Option<&str>)]) -> Self {
            let originals = vars
                .iter()
                .map(|(name, _)| ((*name).to_string(), std::env::var_os(name)))
                .collect::<Vec<_>>();

            for (name, value) in vars {
                if let Some(value) = value {
                    std::env::set_var(name, value);
                } else {
                    std::env::remove_var(name);
                }
            }

            Self { originals }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            for (name, value) in &self.originals {
                if let Some(value) = value {
                    std::env::set_var(name, value);
                } else {
                    std::env::remove_var(name);
                }
            }
        }
    }

    #[test]
    fn trigger_defaults_to_auto_ghost_owner() {
        let config = Config::default();
        assert_eq!(config.trigger.zsh_ghost_owner, ZshGhostOwner::Auto);
    }

    #[test]
    fn parses_zsh_ghost_owner_from_yaml() {
        let yaml = r#"
trigger:
  zsh_ghost_owner: autosuggestions
"#;
        let config: Config = serde_yaml::from_str(yaml).expect("yaml should parse");
        assert_eq!(
            config.trigger.zsh_ghost_owner,
            ZshGhostOwner::Autosuggestions
        );
    }

    #[test]
    fn trigger_defaults_to_message_overlay_backend() {
        let config = Config::default();
        assert_eq!(
            config.trigger.zsh_overlay_backend,
            ZshOverlayBackend::Message
        );
    }

    #[test]
    fn parses_zsh_overlay_backend_from_yaml() {
        let yaml = r#"
trigger:
  zsh_overlay_backend: rprompt
"#;
        let config: Config = serde_yaml::from_str(yaml).expect("yaml should parse");
        assert_eq!(
            config.trigger.zsh_overlay_backend,
            ZshOverlayBackend::Rprompt
        );
    }

    #[test]
    fn deep_merge_keeps_nested_base_values() {
        let base = serde_yaml::from_str::<Value>(
            r#"
context:
  history_window: 20
  priorities:
    history: 80
    plugins: 40
"#,
        )
        .expect("base yaml should parse");
        let override_value = serde_yaml::from_str::<Value>(
            r#"
context:
  priorities:
    plugins: 90
"#,
        )
        .expect("override yaml should parse");

        let merged = Config::deep_merge(base, override_value);
        let config: Config = serde_yaml::from_value(merged).expect("merged config should parse");

        assert_eq!(config.context.history_window, 20);
        assert_eq!(config.context.priorities.history, 80);
        assert_eq!(config.context.priorities.plugins, 90);
    }

    #[test]
    fn load_yaml_layer_skips_empty_document() {
        let file = NamedTempFile::new().expect("temp file should be created");
        std::fs::write(file.path(), "---\n").expect("temp file should be written");

        let layer = Config::load_yaml_layer(file.path(), "test").expect("layer load should work");
        assert!(layer.is_none());
    }

    #[test]
    fn load_yaml_layer_returns_parse_error() {
        let file = NamedTempFile::new().expect("temp file should be created");
        std::fs::write(file.path(), "model: [").expect("temp file should be written");

        let err = Config::load_yaml_layer(file.path(), "test").expect_err("yaml should fail");
        assert!(err.to_string().contains("Failed to parse test config file"));
    }

    #[test]
    fn resolve_override_prefers_nudge_config() {
        let _lock = env_lock().lock().expect("env lock should be acquired");
        let _env = EnvVarGuard::set(&[
            (CONFIG_ENV, Some("/tmp/new.yaml")),
            (LEGACY_CONFIG_ENV, Some("/tmp/legacy.yaml")),
        ]);

        let (env_name, path) =
            Config::resolve_override_config_path().expect("override path should resolve");
        assert_eq!(env_name, CONFIG_ENV);
        assert_eq!(path, PathBuf::from("/tmp/new.yaml"));
    }

    #[test]
    fn validate_rejects_invalid_stale_ratio() {
        let mut config = Config::default();
        config.cache.stale_ratio = 1.2;

        let err = config.validate().expect_err("validation should fail");
        assert!(err.to_string().contains("cache.stale_ratio"));
    }
}
