use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
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
    /// Environment variable containing API key
    pub api_key_env: Option<String>,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434/v1".to_string(),
            model_name: "codellama:7b".to_string(),
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
    /// Load configuration from file or use defaults
    pub fn load() -> Result<Self> {
        // Check for environment variable override
        if let Ok(config_path) = std::env::var("SMARTSHELL_CONFIG") {
            info!("Loading config from SMARTSHELL_CONFIG: {}", config_path);
            return Self::load_from_path(&PathBuf::from(config_path));
        }

        // Use standard config path
        if let Some(config_path) = Self::default_config_path() {
            debug!("Default config path: {}", config_path.display());
            if config_path.exists() {
                info!("Loading config from: {}", config_path.display());
                return Self::load_from_path(&config_path);
            } else {
                debug!("Config file not found at: {}", config_path.display());
            }
        } else {
            warn!("Could not determine default config path (ProjectDirs failed)");
        }

        // Return defaults if no config file exists
        info!("Using default configuration");
        Ok(Self::default())
    }

    /// Load configuration from a specific path
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        debug!("Config file contents ({} bytes):\n{}", contents.len(), contents);

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

    /// Get the default config file path
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
}
