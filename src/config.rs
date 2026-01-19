use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use tracing::{debug, info, warn};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub model: ModelConfig,
    pub context: ContextConfig,
    pub plugins: PluginsConfig,
    pub trigger: TriggerConfig,
    pub privacy: PrivacyConfig,
    pub log: LogConfig,
    pub system_prompt: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: ModelConfig::default(),
            context: ContextConfig::default(),
            plugins: PluginsConfig::default(),
            trigger: TriggerConfig::default(),
            privacy: PrivacyConfig::default(),
            log: LogConfig::default(),
            system_prompt: None,
        }
    }
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginsConfig {
    pub git: GitPluginConfig,
    pub plugin_dir: Option<PathBuf>,
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            git: GitPluginConfig::default(),
            plugin_dir: None,
        }
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitDepth {
    Light,
    Standard,
    Detailed,
}

impl Default for GitDepth {
    fn default() -> Self {
        Self::Standard
    }
}

/// Trigger behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TriggerConfig {
    pub mode: TriggerMode,
    pub hotkey: String,
    pub auto_delay_ms: u64,
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            mode: TriggerMode::Manual,
            hotkey: r"\C-e".to_string(),
            auto_delay_ms: 500,
        }
    }
}

/// Trigger mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerMode {
    Manual,
    Auto,
}

impl Default for TriggerMode {
    fn default() -> Self {
        Self::Manual
    }
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

impl Config {
    /// Load configuration with layered approach:
    /// 1. Start with built-in defaults
    /// 2. Override with config.default.yaml (ships with app, updated on upgrade)
    /// 3. Override with config.yaml (user customizations, preserved on upgrade)
    pub fn load() -> Result<Self> {
        // Check for environment variable override (skips layered loading)
        if let Ok(config_path) = std::env::var("SMARTSHELL_CONFIG") {
            info!("Loading config from SMARTSHELL_CONFIG: {}", config_path);
            return Self::load_from_path(&PathBuf::from(config_path));
        }

        // Start with built-in defaults as YAML value
        let default_config = Self::default();
        let mut merged_value: Value =
            serde_yaml::to_value(&default_config).context("Failed to serialize default config")?;

        // Layer 1: Load config.default.yaml if exists (ships with app)
        if let Some(base_path) = Self::base_config_path() {
            if base_path.exists() {
                debug!("Loading base config from: {}", base_path.display());
                if let Ok(contents) = std::fs::read_to_string(&base_path) {
                    if let Ok(base_value) = serde_yaml::from_str::<Value>(&contents) {
                        merged_value = Self::deep_merge(merged_value, base_value);
                        info!("Base config loaded: {}", base_path.display());
                    } else {
                        warn!("Failed to parse base config: {}", base_path.display());
                    }
                }
            } else {
                debug!("Base config not found: {}", base_path.display());
            }
        }

        // Layer 2: Load config.yaml if exists (user customizations)
        if let Some(user_path) = Self::default_config_path() {
            if user_path.exists() {
                debug!("Loading user config from: {}", user_path.display());
                if let Ok(contents) = std::fs::read_to_string(&user_path) {
                    // Skip empty files
                    let trimmed = contents.trim();
                    if !trimmed.is_empty() && trimmed != "---" {
                        if let Ok(user_value) = serde_yaml::from_str::<Value>(&contents) {
                            merged_value = Self::deep_merge(merged_value, user_value);
                            info!("User config loaded: {}", user_path.display());
                        } else {
                            warn!("Failed to parse user config: {}", user_path.display());
                        }
                    } else {
                        debug!("User config is empty, using defaults");
                    }
                }
            } else {
                debug!("User config not found: {}", user_path.display());
            }
        } else {
            warn!("Could not determine config path (ProjectDirs failed)");
        }

        // Deserialize merged config
        let config: Self =
            serde_yaml::from_value(merged_value).context("Failed to deserialize merged config")?;

        config.validate()?;

        // Log loaded configuration details
        info!("Config loaded successfully (layered):");
        info!("  Model endpoint: {}", config.model.endpoint);
        info!("  Model name: {}", config.model.model_name);
        info!(
            "  API key configured: {}",
            config.model.api_key.is_some() || config.model.api_key_env.is_some()
        );
        info!("  Timeout: {}ms", config.model.timeout_ms);
        debug!("  History window: {}", config.context.history_window);
        debug!("  Git plugin enabled: {}", config.plugins.git.enabled);

        Ok(config)
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
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        debug!(
            "Config file contents ({} bytes):\n{}",
            contents.len(),
            contents
        );

        let config: Self = serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        config.validate()?;

        // Log loaded configuration details
        info!("Config loaded successfully:");
        info!("  Model endpoint: {}", config.model.endpoint);
        info!("  Model name: {}", config.model.model_name);
        info!("  API key env: {:?}", config.model.api_key_env);
        info!("  Timeout: {}ms", config.model.timeout_ms);
        debug!("  History window: {}", config.context.history_window);
        debug!("  Git plugin enabled: {}", config.plugins.git.enabled);

        Ok(config)
    }

    /// Get the base config file path (config.default.yaml - ships with app)
    pub fn base_config_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "nudge").map(|dirs| dirs.config_dir().join("config.default.yaml"))
    }

    /// Get the user config file path (config.yaml - user customizations)
    pub fn default_config_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "nudge").map(|dirs| dirs.config_dir().join("config.yaml"))
    }

    /// Get the socket path for IPC
    /// On Unix: ~/.config/nudge/nudge.sock (Unix Domain Socket)
    /// On Windows: \\.\pipe\nudge_{username} (Named Pipe)
    #[cfg(unix)]
    pub fn socket_path() -> PathBuf {
        ProjectDirs::from("", "", "nudge")
            .map(|dirs| dirs.config_dir().join("nudge.sock"))
            .unwrap_or_else(|| PathBuf::from("/tmp/nudge.sock"))
    }

    /// Get the socket path for IPC (Windows Named Pipe)
    #[cfg(windows)]
    pub fn socket_path() -> PathBuf {
        let username = std::env::var("USERNAME").unwrap_or_else(|_| "default".into());
        PathBuf::from(format!(r"\\.\pipe\nudge_{}", username))
    }

    /// Get the PID file path
    pub fn pid_path() -> PathBuf {
        ProjectDirs::from("", "", "nudge")
            .map(|dirs| dirs.config_dir().join("nudge.pid"))
            .unwrap_or_else(|| {
                let mut temp = std::env::temp_dir();
                temp.push("nudge.pid");
                temp
            })
    }

    /// Get the log directory path (XDG data dir)
    pub fn log_dir() -> PathBuf {
        ProjectDirs::from("", "", "nudge")
            .map(|dirs| dirs.data_dir().join("logs"))
            .unwrap_or_else(|| {
                let mut temp = std::env::temp_dir();
                temp.push("nudge");
                temp.push("logs");
                temp
            })
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
            let has_direct_key = self
                .model
                .api_key
                .as_ref()
                .is_some_and(|k| !k.is_empty());

            // Check if api_key_env is set and the env var exists
            let has_env_key = self
                .model
                .api_key_env
                .as_ref()
                .is_some_and(|env_var| !env_var.is_empty() && std::env::var(env_var).is_ok());

            if !has_direct_key && !has_env_key {
                let config_path = Self::default_config_path()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "config file".to_string());

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
