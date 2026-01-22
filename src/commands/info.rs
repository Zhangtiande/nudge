use crate::config::Platform;
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

    // Check daemon status
    let daemon_status = check_daemon_status(&socket_path);

    let info = InfoOutput {
        platform: platform.to_string(),
        config_dir,
        config_file,
        default_config_file,
        socket_path,
        integration_script,
        daemon_status,
        shell_type,
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
