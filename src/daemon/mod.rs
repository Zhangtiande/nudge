pub mod server;
pub mod session;
pub mod llm;
pub mod context;
pub mod plugins;
pub mod sanitizer;
pub mod safety;

use std::fs;
use std::process::Command;

use anyhow::{Context, Result};
use tracing::{info, warn};

use crate::config::Config;

/// Run the daemon
pub async fn run(foreground: bool, fork: bool) -> Result<()> {
    let config = Config::load()?;

    // Ensure config directory exists
    if let Some(config_dir) = Config::socket_path().parent() {
        fs::create_dir_all(config_dir)?;
    }

    if fork && !foreground {
        // Fork into background
        info!("Forking daemon into background...");
        fork_daemon()?;
        return Ok(());
    }

    // Write PID file
    let pid = std::process::id();
    fs::write(Config::pid_path(), pid.to_string())?;

    info!("Starting Nudge daemon (pid: {})", pid);
    info!("Socket path: {}", Config::socket_path().display());

    // Run the server
    let result = server::run(config).await;

    // Cleanup PID file
    let _ = fs::remove_file(Config::pid_path());

    result
}

/// Fork the process and run daemon in background
fn fork_daemon() -> Result<()> {
    let exe = std::env::current_exe()?;
    
    Command::new(exe)
        .arg("daemon")
        .arg("--foreground")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("Failed to fork daemon")?;

    Ok(())
}

/// Start daemon in background
pub async fn start() -> Result<()> {
    // Check if already running
    if is_running() {
        println!("Nudge daemon is already running");
        return Ok(());
    }

    fork_daemon()?;
    println!("Nudge daemon started");
    Ok(())
}

/// Stop running daemon
pub async fn stop() -> Result<()> {
    let pid_path = Config::pid_path();

    if !pid_path.exists() {
        println!("Nudge daemon is not running");
        return Ok(());
    }

    let pid_str = fs::read_to_string(&pid_path)?;
    let pid: i32 = pid_str.trim().parse()?;

    // Send SIGTERM
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        if let Err(e) = kill(Pid::from_raw(pid), Signal::SIGTERM) {
            warn!("Failed to send SIGTERM: {}", e);
        }
    }

    // Remove PID file
    let _ = fs::remove_file(&pid_path);
    
    // Remove socket file
    let _ = fs::remove_file(Config::socket_path());

    println!("Nudge daemon stopped");
    Ok(())
}

/// Check daemon status
pub async fn status() -> Result<()> {
    if is_running() {
        let pid_str = fs::read_to_string(Config::pid_path())?;
        println!("Nudge daemon is running (pid: {})", pid_str.trim());
    } else {
        println!("Nudge daemon is not running");
    }
    Ok(())
}

/// Check if daemon is running
fn is_running() -> bool {
    let pid_path = Config::pid_path();
    let socket_path = Config::socket_path();

    if !pid_path.exists() || !socket_path.exists() {
        return false;
    }

    // Check if PID is still alive
    if let Ok(pid_str) = fs::read_to_string(&pid_path) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;
                // Signal 0 checks if process exists
                return kill(Pid::from_raw(pid), Signal::SIGCONT).is_ok();
            }
            #[cfg(not(unix))]
            {
                return true;
            }
        }
    }

    false
}
