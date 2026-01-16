pub mod context;
pub mod llm;
pub mod plugins;
pub mod safety;
pub mod sanitizer;
pub mod server;
pub mod session;

use std::fs;
use std::process::Command;

use anyhow::{Context, Result};
use tracing::{debug, info, warn};

use crate::config::Config;

/// Run the daemon
pub async fn run(foreground: bool, fork: bool) -> Result<()> {
    debug!("Loading configuration...");
    let config = Config::load()?;
    
    // Validate LLM configuration
    if let Err(e) = config.validate_llm_config() {
        eprintln!("\n\x1b[1;31mError: LLM configuration issue\x1b[0m\n");
        eprintln!("{}", e);
        eprintln!();
        anyhow::bail!("Cannot start daemon without valid LLM configuration");
    }

    // Ensure config directory exists (Unix only - socket is a filesystem path)
    // On Windows, Named Pipes don't need directory creation
    #[cfg(unix)]
    if let Some(config_dir) = Config::socket_path().parent() {
        fs::create_dir_all(config_dir)?;
    }

    if fork && !foreground {
        // Fork into background
        info!("Forking daemon into background...");
        fork_daemon()?;
        return Ok(());
    }

    // Write PID file (ensure directory exists first)
    let pid_path = Config::pid_path();
    if let Some(pid_dir) = pid_path.parent() {
        fs::create_dir_all(pid_dir)?;
    }
    let pid = std::process::id();
    fs::write(&pid_path, pid.to_string())?;

    info!("Starting Nudge daemon (pid: {})", pid);
    info!("Socket path: {}", Config::socket_path().display());
    debug!("PID file: {}", pid_path.display());

    // Run the server
    let result = server::run(config).await;

    // Cleanup PID file
    let _ = fs::remove_file(Config::pid_path());

    result
}

/// Fork the process and run daemon in background
fn fork_daemon() -> Result<()> {
    let exe = std::env::current_exe()?;

    #[cfg(unix)]
    {
        Command::new(exe)
            .arg("daemon")
            .arg("--foreground")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .context("Failed to fork daemon")?;
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;

        Command::new(exe)
            .arg("daemon")
            .arg("--foreground")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
            .spawn()
            .context("Failed to fork daemon")?;
    }

    Ok(())
}

/// Validate LLM configuration before starting daemon
fn validate_config_before_start() -> Result<Config> {
    let config = Config::load()?;
    
    // Validate LLM configuration
    if let Err(e) = config.validate_llm_config() {
        eprintln!("\n\x1b[1;31mError: LLM configuration issue\x1b[0m\n");
        eprintln!("{}", e);
        eprintln!();
        anyhow::bail!("Cannot start daemon without valid LLM configuration");
    }
    
    Ok(config)
}

/// Start daemon in background
pub async fn start() -> Result<()> {
    // Validate configuration first
    let config = validate_config_before_start()?;
    
    // Check if already running
    if is_running() {
        println!("Nudge daemon is already running");
        return Ok(());
    }

    // Print LLM configuration summary
    println!("\x1b[1;32mLLM Configuration:\x1b[0m");
    println!("{}", config.llm_config_summary());
    println!();

    fork_daemon()?;
    println!("\x1b[1;32mNudge daemon started\x1b[0m");
    Ok(())
}

/// Restart the daemon (stop + start)
pub async fn restart() -> Result<()> {
    println!("Restarting Nudge daemon...");
    
    // Validate config before stopping (fail fast)
    let config = validate_config_before_start()?;
    
    // Stop if running
    let was_running = is_running();
    if was_running {
        stop().await?;
        // Wait a bit for the process to fully terminate
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    
    // Print LLM configuration summary
    println!("\x1b[1;32mLLM Configuration:\x1b[0m");
    println!("{}", config.llm_config_summary());
    println!();
    
    // Start
    fork_daemon()?;
    println!("\x1b[1;32mNudge daemon restarted\x1b[0m");
    Ok(())
}

/// Stop running daemon
pub async fn stop() -> Result<()> {
    let pid_path = Config::pid_path();
    #[cfg(unix)]
    let socket_path = Config::socket_path();

    if !pid_path.exists() {
        // Clean up any stale socket file (Unix only, Windows Named Pipes don't leave files)
        #[cfg(unix)]
        if socket_path.exists() {
            let _ = fs::remove_file(&socket_path);
            println!("Cleaned up stale socket file");
        }
        println!("Nudge daemon is not running");
        return Ok(());
    }

    let pid_str = fs::read_to_string(&pid_path)?;
    let pid: u32 = pid_str.trim().parse()?;

    // Check if process is actually running before attempting to stop
    let process_exists = is_process_alive(pid);

    if process_exists {
        if terminate_process(pid) {
            println!("Nudge daemon stopped (pid: {})", pid);
        } else {
            warn!("Failed to terminate daemon process");
        }
    } else {
        println!("Daemon process not found (stale pid file), cleaning up...");
    }

    // Always clean up files
    let _ = fs::remove_file(&pid_path);
    #[cfg(unix)]
    let _ = fs::remove_file(&socket_path);

    Ok(())
}

/// Check if a process with given PID is alive (Unix implementation)
#[cfg(unix)]
fn is_process_alive(pid: u32) -> bool {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;
    kill(Pid::from_raw(pid as i32), Signal::SIGCONT).is_ok()
}

/// Check if a process with given PID is alive (Windows implementation)
#[cfg(windows)]
fn is_process_alive(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if !handle.is_null() {
            CloseHandle(handle);
            true
        } else {
            false
        }
    }
}

/// Terminate a process by PID (Unix implementation)
#[cfg(unix)]
fn terminate_process(pid: u32) -> bool {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;
    kill(Pid::from_raw(pid as i32), Signal::SIGTERM).is_ok()
}

/// Terminate a process by PID (Windows implementation)
#[cfg(windows)]
fn terminate_process(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if !handle.is_null() {
            let result = TerminateProcess(handle, 0) != 0;
            CloseHandle(handle);
            result
        } else {
            false
        }
    }
}

/// Check daemon status
pub async fn status() -> Result<()> {
    let (running, pid) = is_running_with_cleanup();
    if running {
        println!("Nudge daemon is running (pid: {})", pid);
    } else {
        println!("Nudge daemon is not running");
    }
    Ok(())
}

/// Check if daemon is running, and clean up stale files if not
fn is_running_with_cleanup() -> (bool, u32) {
    let pid_path = Config::pid_path();
    #[cfg(unix)]
    let socket_path = Config::socket_path();

    if !pid_path.exists() {
        // No PID file, clean up stale socket if exists (Unix only)
        #[cfg(unix)]
        if socket_path.exists() {
            let _ = fs::remove_file(&socket_path);
        }
        return (false, 0);
    }

    // Check if PID is still alive
    if let Ok(pid_str) = fs::read_to_string(&pid_path) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            if is_process_alive(pid) {
                return (true, pid);
            }
        }
    }

    // Process not running, clean up stale files
    let _ = fs::remove_file(&pid_path);
    #[cfg(unix)]
    let _ = fs::remove_file(&socket_path);
    (false, 0)
}

/// Check if daemon is running (simple version for start command)
fn is_running() -> bool {
    is_running_with_cleanup().0
}
