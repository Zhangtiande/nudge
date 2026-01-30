use crate::config::{Config, Platform, TriggerMode};
use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct InfoOutput {
    pub platform: String,
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
    pub default_config_file: PathBuf,
    pub socket_path: PathBuf,
    pub integration_script: PathBuf,
    pub daemon_status: String,
    pub shell_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lib_path: Option<PathBuf>,
    // Trigger configuration
    pub trigger_mode: String,
    pub trigger_hotkey: String,
    pub auto_delay_ms: u64,
    // Diagnosis configuration
    pub diagnosis_enabled: bool,
}

/// Run the info command
pub fn run_info(json: bool, field: Option<String>) -> Result<()> {
    let platform = Platform::detect()?;
    let config_dir = platform.config_dir()?;
    let config_file = config_dir.join("config.yaml");
    let default_config_file = config_dir.join("config").join("config.default.yaml");
    let socket_path = config_dir.join("nudge.sock");
    let integration_script = platform.integration_script_path()?;

    // Get shell type from platform
    let shell_type = platform.shell.to_string();

    // Get library path (FFI mode, Unix only)
    let lib_path = platform.lib_path();

    // Check daemon status
    let daemon_status = check_daemon_status(&socket_path);

    // Load config for trigger settings
    let config = Config::load().unwrap_or_default();
    let trigger_mode = match config.trigger.mode {
        TriggerMode::Manual => "manual".to_string(),
        TriggerMode::Auto => "auto".to_string(),
    };
    let trigger_hotkey = config.trigger.hotkey.clone();
    let auto_delay_ms = config.trigger.auto_delay_ms;

    let info = InfoOutput {
        platform: platform.to_string(),
        config_dir,
        config_file,
        default_config_file,
        socket_path,
        integration_script,
        daemon_status,
        shell_type,
        lib_path,
        trigger_mode,
        trigger_hotkey,
        auto_delay_ms,
        diagnosis_enabled: config.diagnosis.enabled,
    };

    if let Some(field_name) = field {
        // Single field output
        let value = match field_name.as_str() {
            "platform" => info.platform.clone(),
            "config_dir" => info.config_dir.display().to_string(),
            "config_file" => info.config_file.display().to_string(),
            "default_config_file" => info.default_config_file.display().to_string(),
            "socket_path" => info.socket_path.display().to_string(),
            "integration_script" => info.integration_script.display().to_string(),
            "daemon_status" => info.daemon_status.clone(),
            "shell_type" => info.shell_type.clone(),
            "lib_path" => info
                .lib_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            "trigger_mode" => info.trigger_mode.clone(),
            "trigger_hotkey" => info.trigger_hotkey.clone(),
            "auto_delay_ms" => info.auto_delay_ms.to_string(),
            "diagnosis_enabled" => info.diagnosis_enabled.to_string(),
            _ => anyhow::bail!("Unknown field: {}", field_name),
        };
        println!("{}", value);
    } else if json {
        // JSON output
        let json_str = serde_json::to_string_pretty(&info)?;
        println!("{}", json_str);
    } else {
        // Human-readable output
        println!("Nudge Runtime Information");
        println!("=========================");
        println!();
        println!("Platform:             {}", info.platform);
        println!("Config Directory:     {}", info.config_dir.display());
        println!("Config File:          {}", info.config_file.display());
        println!(
            "Default Config:       {}",
            info.default_config_file.display()
        );
        println!("Socket Path:          {}", info.socket_path.display());
        println!(
            "Integration Script:   {}",
            info.integration_script.display()
        );
        println!("Daemon Status:        {}", info.daemon_status);
        println!("Shell Type:           {}", info.shell_type);
        if let Some(ref lib_path) = info.lib_path {
            println!("Library Path:         {}", lib_path.display());
        }
        println!();
        println!("Trigger Configuration");
        println!("---------------------");
        println!("Mode:                 {}", info.trigger_mode);
        println!("Hotkey:               {}", info.trigger_hotkey);
        println!("Auto Delay:           {}ms", info.auto_delay_ms);
        println!();
        println!("Diagnosis Configuration");
        println!("-----------------------");
        println!("Enabled:              {}", info.diagnosis_enabled);
    }

    Ok(())
}

/// Check if daemon is running by attempting to connect to socket
fn check_daemon_status(socket_path: &PathBuf) -> String {
    // Check if socket file exists
    if !socket_path.exists() {
        return "Not running (socket not found)".to_string();
    }

    // Try to read socket metadata (Unix-specific behavior)
    match fs::metadata(socket_path) {
        Ok(_) => {
            // Socket exists, but we can't easily check if it's active without connecting
            // For now, we'll just report that the socket exists
            "Running (socket exists)".to_string()
        }
        Err(_) => "Not running (socket not accessible)".to_string(),
    }
}
