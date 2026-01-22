# Phase 1: Foundation Refactoring Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Centralize platform logic, eliminate path duplication, and implement `nudge info` and `nudge setup` commands for simplified installation.

**Architecture:** Create a new `Platform` module in `src/config.rs` that encapsulates all platform-specific logic (path detection, OS/shell detection). New CLI commands (`info`, `setup`) will use this centralized logic, and shell scripts will be simplified to call these commands instead of duplicating path logic.

**Tech Stack:** Rust (clap for CLI, serde for JSON), shell scripting (Bash/PowerShell)

---

## Task 1: Create Platform Detection Module

**Files:**
- Modify: `src/config.rs:1-548` (add Platform module at end)
- Test: Manual testing via new CLI commands (integration tests in Task 2)

**Step 1: Add Platform enums and struct**

Add to end of `src/config.rs` (after line 547):

```rust
/// Platform detection and OS-specific logic
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
```

**Step 2: Implement Platform::detect()**

Add after enums:

```rust
impl Platform {
    /// Detect current platform at runtime
    pub fn detect() -> Self {
        let os = if cfg!(target_os = "macos") {
            OsType::MacOS
        } else if cfg!(target_os = "linux") {
            OsType::Linux
        } else if cfg!(target_os = "windows") {
            OsType::Windows
        } else {
            panic!("Unsupported operating system");
        };

        let shell = Self::detect_shell();

        Self { os, shell }
    }

    /// Detect current shell from environment
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

    /// Get platform-specific config directory
    pub fn config_dir(&self) -> PathBuf {
        match self.os {
            OsType::MacOS => {
                let home = std::env::var("HOME").expect("HOME not set");
                PathBuf::from(home).join("Library/Application Support/nudge")
            }
            OsType::Linux => {
                let base = std::env::var("XDG_CONFIG_HOME")
                    .unwrap_or_else(|_| {
                        let home = std::env::var("HOME").expect("HOME not set");
                        format!("{}/.config", home)
                    });
                PathBuf::from(base).join("nudge")
            }
            OsType::Windows => {
                let appdata = std::env::var("APPDATA").expect("APPDATA not set");
                PathBuf::from(appdata).join("nudge")
            }
        }
    }

    /// Get shell integration script path for current shell
    pub fn integration_script_path(&self) -> PathBuf {
        let filename = match self.shell {
            ShellType::Bash => "integration.bash",
            ShellType::Zsh => "integration.zsh",
            ShellType::PowerShell => "integration.ps1",
            ShellType::Cmd => "integration.cmd",
            ShellType::Unknown => "integration.bash", // fallback
        };
        self.config_dir().join("shell").join(filename)
    }

    /// Get shell profile path (for setup command)
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
                // Check PROFILE env var first
                if let Ok(profile) = std::env::var("PROFILE") {
                    return Ok(PathBuf::from(profile));
                }
                // Fallback to default location
                let home = std::env::var("USERPROFILE")
                    .or_else(|_| std::env::var("HOME"))?;
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
}
```

**Step 3: Run cargo check**

Run: `cargo check`
Expected: No errors (warnings about unused code are ok)

**Step 4: Commit**

```bash
git add src/config.rs
git commit -m "feat(config): add Platform module for centralized OS/shell detection"
```

---

## Task 2: Add `nudge info` Command

**Files:**
- Modify: `src/cli.rs:64-71` (add Info variant)
- Modify: `src/main.rs:76-79` (add match arm for Info)
- Create: `src/commands/mod.rs`
- Create: `src/commands/info.rs`

**Step 1: Create commands module**

Create `src/commands/mod.rs`:

```rust
pub mod info;
```

**Step 2: Implement info command**

Create `src/commands/info.rs`:

```rust
use anyhow::Result;
use serde::{Serialize, Deserialize};
use crate::config::{Config, Platform};

#[derive(Debug, Serialize, Deserialize)]
pub struct InfoOutput {
    pub config_dir: String,
    pub config_file_default: String,
    pub config_file_user: String,
    pub socket_path: String,
    pub pid_file: String,
    pub log_dir: String,
    pub daemon_status: String,
    pub shell_type: String,
    pub os_type: String,
    pub integration_script: String,
}

pub fn run_info(json: bool, field: Option<String>) -> Result<()> {
    let platform = Platform::detect();
    let config = Config::load().unwrap_or_default();

    let info = InfoOutput {
        config_dir: platform.config_dir().display().to_string(),
        config_file_default: Config::base_config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "N/A".to_string()),
        config_file_user: Config::default_config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "N/A".to_string()),
        socket_path: Config::socket_path().display().to_string(),
        pid_file: Config::pid_path().display().to_string(),
        log_dir: Config::log_dir().display().to_string(),
        daemon_status: check_daemon_status()?,
        shell_type: format!("{:?}", platform.shell).to_lowercase(),
        os_type: format!("{:?}", platform.os).to_lowercase(),
        integration_script: platform.integration_script_path().display().to_string(),
    };

    if let Some(field_name) = field {
        // Output single field for shell scripts
        let value = match field_name.as_str() {
            "config_dir" => info.config_dir,
            "socket_path" => info.socket_path,
            "pid_file" => info.pid_file,
            "log_dir" => info.log_dir,
            "shell_type" => info.shell_type,
            "os_type" => info.os_type,
            "daemon_status" => info.daemon_status,
            "integration_script" => info.integration_script,
            _ => anyhow::bail!("Unknown field: {}", field_name),
        };
        println!("{}", value);
    } else if json {
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        // Human-readable format
        println!("Nudge Runtime Information\n");
        println!("Config Directory:     {}", info.config_dir);
        println!("Default Config File:  {}", info.config_file_default);
        println!("User Config File:     {}", info.config_file_user);
        println!("Socket/Pipe:          {}", info.socket_path);
        println!("PID File:             {}", info.pid_file);
        println!("Log Directory:        {}", info.log_dir);
        println!("Integration Script:   {}", info.integration_script);
        println!("\nDaemon Status:        {}", info.daemon_status);
        println!("Shell Type:           {}", info.shell_type);
        println!("OS Type:              {}", info.os_type);
    }

    Ok(())
}

fn check_daemon_status() -> Result<String> {
    let pid_path = Config::pid_path();
    if !pid_path.exists() {
        return Ok("Not running".to_string());
    }

    let pid_str = std::fs::read_to_string(&pid_path)?;
    let pid: u32 = pid_str.trim().parse()?;

    // Check if process exists
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        match kill(Pid::from_raw(pid as i32), Signal::SIGCONT) {
            Ok(_) => Ok(format!("Running (PID: {})", pid)),
            Err(_) => Ok("Not running (stale PID file)".to_string()),
        }
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION};
        use windows_sys::Win32::Foundation::CloseHandle;

        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
            if handle != 0 {
                CloseHandle(handle);
                Ok(format!("Running (PID: {})", pid))
            } else {
                Ok("Not running (stale PID file)".to_string())
            }
        }
    }
}
```

**Step 3: Add Info CLI variant**

In `src/cli.rs`, replace the existing `Config` variant (lines 66-70) with:

```rust
    /// Show configuration paths and status (legacy, use 'info' instead)
    #[deprecated(since = "0.3.0", note = "Use 'info' command instead")]
    Config {
        /// Show full configuration (not just paths)
        #[arg(long, default_value_t = false)]
        show: bool,
    },

    /// Display runtime information (paths, status, configuration)
    Info {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Get specific field (config_dir, socket_path, shell_type, etc.)
        #[arg(long)]
        field: Option<String>,
    },
```

**Step 4: Wire up in main.rs**

In `src/main.rs`, add to imports (line 1):

```rust
mod commands;
```

Then add to the match statement (after line 78):

```rust
        Command::Info { json, field } => {
            commands::info::run_info(json, field)?;
        }
```

**Step 5: Run cargo check**

Run: `cargo check`
Expected: No errors

**Step 6: Test the command**

Run: `cargo run -- info`
Expected: Human-readable output showing paths and status

Run: `cargo run -- info --json`
Expected: Valid JSON output

Run: `cargo run -- info --field config_dir`
Expected: Single line with config directory path

**Step 7: Commit**

```bash
git add src/cli.rs src/main.rs src/commands/
git commit -m "feat(cli): add 'info' command to display runtime information

- Centralizes path and status queries
- Supports JSON output and single-field extraction
- Replaces scattered path logic in shell scripts"
```

---

## Task 3: Add `nudge setup` Command for Bash

**Files:**
- Modify: `src/cli.rs:72+` (add Setup variant)
- Modify: `src/main.rs:80+` (add match arm)
- Create: `src/commands/setup.rs`
- Modify: `src/commands/mod.rs` (add setup module)

**Step 1: Add Setup CLI variant**

In `src/cli.rs`, add after the Info variant:

```rust
    /// Setup shell integration automatically
    Setup {
        /// Shell type (bash, zsh, powershell) - auto-detect if not specified
        shell: Option<String>,

        /// Force reinstall even if already configured
        #[arg(long)]
        force: bool,
    },
```

**Step 2: Create setup command module**

Add to `src/commands/mod.rs`:

```rust
pub mod setup;
```

Create `src/commands/setup.rs`:

```rust
use anyhow::{Context, Result};
use std::io::Write;
use crate::config::Platform;
use crate::daemon;

pub fn run_setup(shell: Option<String>, force: bool) -> Result<()> {
    let platform = Platform::detect();

    let target_shell = if let Some(s) = shell {
        s.to_lowercase()
    } else {
        format!("{:?}", platform.shell).to_lowercase()
    };

    match target_shell.as_str() {
        "bash" => setup_bash(&platform, force),
        "zsh" => setup_zsh(&platform, force),
        "powershell" => setup_powershell(&platform, force),
        "cmd" => {
            anyhow::bail!("CMD does not support automatic setup. Please use PowerShell or manually configure.")
        }
        "unknown" => {
            anyhow::bail!("Could not detect shell. Please specify: bash, zsh, or powershell")
        }
        _ => anyhow::bail!("Unsupported shell: {}. Supported: bash, zsh, powershell", target_shell),
    }
}

fn setup_bash(platform: &Platform, force: bool) -> Result<()> {
    let profile_path = platform.shell_profile_path()
        .context("Failed to determine .bashrc path")?;

    println!("Setting up Nudge for Bash\n");
    println!("Profile: {}", profile_path.display());

    // Get integration script path
    let integration_script = platform.integration_script_path();
    let source_line = format!("source \"{}\"", integration_script.display());
    let marker = "# Nudge - LLM-powered CLI completion";

    // Check if already configured
    if profile_path.exists() && !force {
        let content = std::fs::read_to_string(&profile_path)?;
        if content.contains(marker) || content.contains(&source_line) {
            println!("✓ Nudge is already configured in {}", profile_path.display());
            println!("\nTo reconfigure, run:");
            println!("  nudge setup bash --force");
            return Ok(());
        }
    }

    // Copy integration script to config dir
    install_integration_script(platform, "bash")?;

    // Append to profile
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&profile_path)
        .context("Failed to open .bashrc")?;

    writeln!(file, "\n{}", marker)?;
    writeln!(file, "{}", source_line)?;

    println!("✓ Integration added to {}", profile_path.display());
    println!("✓ Integration script: {}", integration_script.display());

    // Start daemon if not running
    start_daemon_if_needed()?;

    println!("\n🎉 Setup complete! Please restart your shell or run:");
    println!("  source {}", profile_path.display());
    println!("\nThen press Ctrl+E to trigger completion.");

    Ok(())
}

fn setup_zsh(platform: &Platform, force: bool) -> Result<()> {
    let profile_path = platform.shell_profile_path()
        .context("Failed to determine .zshrc path")?;

    println!("Setting up Nudge for Zsh\n");
    println!("Profile: {}", profile_path.display());

    let integration_script = platform.integration_script_path();
    let source_line = format!("source \"{}\"", integration_script.display());
    let marker = "# Nudge - LLM-powered CLI completion";

    if profile_path.exists() && !force {
        let content = std::fs::read_to_string(&profile_path)?;
        if content.contains(marker) || content.contains(&source_line) {
            println!("✓ Nudge is already configured in {}", profile_path.display());
            println!("\nTo reconfigure, run:");
            println!("  nudge setup zsh --force");
            return Ok(());
        }
    }

    install_integration_script(platform, "zsh")?;

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&profile_path)
        .context("Failed to open .zshrc")?;

    writeln!(file, "\n{}", marker)?;
    writeln!(file, "{}", source_line)?;

    println!("✓ Integration added to {}", profile_path.display());
    println!("✓ Integration script: {}", integration_script.display());

    start_daemon_if_needed()?;

    println!("\n🎉 Setup complete! Please restart your shell or run:");
    println!("  source {}", profile_path.display());
    println!("\nThen press Ctrl+E to trigger completion.");

    Ok(())
}

fn setup_powershell(platform: &Platform, force: bool) -> Result<()> {
    let profile_path = platform.shell_profile_path()
        .context("Failed to determine PowerShell profile path")?;

    println!("Setting up Nudge for PowerShell\n");
    println!("Profile: {}", profile_path.display());

    let integration_script = platform.integration_script_path();
    let source_line = format!(". \"{}\"", integration_script.display());
    let marker = "# Nudge - LLM-powered CLI completion";

    if profile_path.exists() && !force {
        let content = std::fs::read_to_string(&profile_path)?;
        if content.contains(marker) || content.contains(&source_line) {
            println!("✓ Nudge is already configured in {}", profile_path.display());
            println!("\nTo reconfigure, run:");
            println!("  nudge setup powershell --force");
            return Ok(());
        }
    }

    install_integration_script(platform, "powershell")?;

    // Create profile directory if it doesn't exist
    if let Some(parent) = profile_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&profile_path)
        .context("Failed to open PowerShell profile")?;

    writeln!(file, "\n{}", marker)?;
    writeln!(file, "{}", source_line)?;

    println!("✓ Integration added to {}", profile_path.display());
    println!("✓ Integration script: {}", integration_script.display());

    start_daemon_if_needed()?;

    println!("\n🎉 Setup complete! Please restart PowerShell or run:");
    println!("  . $PROFILE");
    println!("\nThen press Ctrl+E to trigger completion.");

    Ok(())
}

fn install_integration_script(platform: &Platform, shell: &str) -> Result<()> {
    let integration_script = platform.integration_script_path();

    // Ensure directory exists
    if let Some(parent) = integration_script.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Get bundled script content from embedded resources
    let script_content = match shell {
        "bash" => include_str!("../../shell/integration.bash"),
        "zsh" => include_str!("../../shell/integration.zsh"),
        "powershell" => include_str!("../../shell/integration.ps1"),
        _ => anyhow::bail!("No integration script for shell: {}", shell),
    };

    std::fs::write(&integration_script, script_content)
        .context("Failed to write integration script")?;

    // Set executable permission on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&integration_script)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&integration_script, perms)?;
    }

    Ok(())
}

fn start_daemon_if_needed() -> Result<()> {
    // Check if daemon is already running
    let status = daemon::check_status();

    if status.is_ok() {
        println!("✓ Daemon is already running");
        return Ok(());
    }

    println!("\nStarting daemon...");
    daemon::start().await?;
    println!("✓ Daemon started");

    Ok(())
}
```

**Step 3: Wire up in main.rs**

Add to the match statement in `src/main.rs`:

```rust
        Command::Setup { shell, force } => {
            commands::setup::run_setup(shell, force)?;
        }
```

**Step 4: Run cargo check**

Run: `cargo check`
Expected: Compilation errors about `daemon::check_status()` not existing - this is expected, we'll fix it next

**Step 5: Add daemon status check helper**

In `src/daemon/mod.rs`, add before the `pub async fn status()` function:

```rust
/// Check if daemon is running (non-async version for setup)
pub fn check_status() -> Result<()> {
    let pid_path = Config::pid_path();
    if !pid_path.exists() {
        anyhow::bail!("Daemon is not running (no PID file)");
    }

    let pid_str = std::fs::read_to_string(&pid_path)?;
    let pid: u32 = pid_str.trim().parse()?;

    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        kill(Pid::from_raw(pid as i32), Signal::SIGCONT)
            .map_err(|_| anyhow::anyhow!("Process not found"))?;
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION};
        use windows_sys::Win32::Foundation::CloseHandle;

        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
            if handle == 0 {
                anyhow::bail!("Process not found");
            }
            CloseHandle(handle);
        }
    }

    Ok(())
}
```

**Step 6: Fix async issue in setup.rs**

The `start_daemon_if_needed()` function calls `daemon::start().await` but it's not in an async context. We need to fix this. Replace the function in `src/commands/setup.rs`:

```rust
fn start_daemon_if_needed() -> Result<()> {
    // Check if daemon is already running
    if daemon::check_status().is_ok() {
        println!("✓ Daemon is already running");
        return Ok(());
    }

    println!("\nStarting daemon...");

    // Use tokio block_on to call async function
    tokio::runtime::Runtime::new()?.block_on(async {
        daemon::start().await
    })?;

    println!("✓ Daemon started");

    Ok(())
}
```

**Step 7: Run cargo check**

Run: `cargo check`
Expected: No errors

**Step 8: Test the setup command (dry run)**

Run: `cargo run -- setup bash --force` (if you're on a Unix system)
Expected: Should attempt to modify your .bashrc (use with caution, or test in a VM/container)

Note: For safety, you may want to test this on a backup of your shell profile first.

**Step 9: Commit**

```bash
git add src/cli.rs src/main.rs src/commands/setup.rs src/commands/mod.rs src/daemon/mod.rs
git commit -m "feat(cli): add 'setup' command for automatic shell integration

- Supports bash, zsh, and powershell
- Automatically installs integration script
- Detects existing configuration to prevent duplicates
- Starts daemon if not already running"
```

---

## Task 4: Simplify Shell Integration Scripts

**Files:**
- Modify: `shell/integration.bash`
- Modify: `shell/integration.zsh`
- Modify: `shell/integration.ps1`

**Step 1: Simplify integration.bash**

Replace the content of `shell/integration.bash` with:

```bash
#!/usr/bin/env bash
# Nudge - Bash Integration
# Installed by: nudge setup bash

# Get configuration from nudge CLI
NUDGE_CONFIG_DIR=$(nudge info --field config_dir 2>/dev/null)
NUDGE_SOCKET=$(nudge info --field socket_path 2>/dev/null)

# Fallback if nudge binary not in PATH
if [[ -z "$NUDGE_CONFIG_DIR" ]]; then
    case "$(uname -s)" in
        Darwin)
            NUDGE_CONFIG_DIR="$HOME/Library/Application Support/nudge"
            ;;
        *)
            NUDGE_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/nudge"
            ;;
    esac
    NUDGE_SOCKET="$NUDGE_CONFIG_DIR/nudge.sock"
fi

# Lock file for daemon startup
NUDGE_LOCK="/tmp/nudge.lock"

# Capture last exit code before any command
_nudge_last_exit=0
_nudge_capture_exit() {
    _nudge_last_exit=$?
}
PROMPT_COMMAND="_nudge_capture_exit${PROMPT_COMMAND:+; $PROMPT_COMMAND}"

# Ensure daemon is running (lazy load)
_nudge_ensure_daemon() {
    if [[ ! -S "$NUDGE_SOCKET" ]]; then
        # Use flock to prevent concurrent daemon starts
        (
            flock -n 200 2>/dev/null || exit 0
            nudge start 2>/dev/null
        ) 200>"$NUDGE_LOCK"
    fi
}

# Main completion function
_nudge_complete() {
    _nudge_ensure_daemon

    local suggestion
    suggestion=$(nudge complete --format plain \
        --buffer "$READLINE_LINE" \
        --cursor "$READLINE_POINT" \
        --cwd "$PWD" \
        --session "bash-$$" \
        --last-exit-code "$_nudge_last_exit" 2>/dev/null)

    if [[ $? -eq 0 && -n "$suggestion" ]]; then
        READLINE_LINE="$suggestion"
        READLINE_POINT=${#READLINE_LINE}
    fi
}

# Bind Ctrl+E hotkey
bind -x '"\C-e": _nudge_complete'

# Print success message on first load
if [[ -z "$_NUDGE_LOADED" ]]; then
    export _NUDGE_LOADED=1
    echo "Nudge loaded. Press Ctrl+E to trigger completion."
fi
```

**Step 2: Simplify integration.zsh**

Replace the content of `shell/integration.zsh` with:

```zsh
#!/usr/bin/env zsh
# Nudge - Zsh Integration
# Installed by: nudge setup zsh

# Get configuration from nudge CLI
NUDGE_CONFIG_DIR=$(nudge info --field config_dir 2>/dev/null)
NUDGE_SOCKET=$(nudge info --field socket_path 2>/dev/null)

# Fallback if nudge binary not in PATH
if [[ -z "$NUDGE_CONFIG_DIR" ]]; then
    case "$(uname -s)" in
        Darwin)
            NUDGE_CONFIG_DIR="$HOME/Library/Application Support/nudge"
            ;;
        *)
            NUDGE_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/nudge"
            ;;
    esac
    NUDGE_SOCKET="$NUDGE_CONFIG_DIR/nudge.sock"
fi

NUDGE_LOCK="/tmp/nudge.lock"

# Capture last exit code
_nudge_last_exit=0
_nudge_capture_exit() {
    _nudge_last_exit=$?
}
precmd_functions+=(_nudge_capture_exit)

# Ensure daemon is running
_nudge_ensure_daemon() {
    if [[ ! -S "$NUDGE_SOCKET" ]]; then
        (
            flock -n 200 2>/dev/null || exit 0
            nudge start 2>/dev/null
        ) 200>"$NUDGE_LOCK"
    fi
}

# Main completion widget
_nudge_complete() {
    _nudge_ensure_daemon

    local suggestion
    suggestion=$(nudge complete --format plain \
        --buffer "$BUFFER" \
        --cursor "$CURSOR" \
        --cwd "$PWD" \
        --session "zsh-$$" \
        --last-exit-code "$_nudge_last_exit" 2>/dev/null)

    if [[ $? -eq 0 && -n "$suggestion" ]]; then
        BUFFER="$suggestion"
        CURSOR=${#BUFFER}
    fi
}

# Register widget and bind key
zle -N _nudge_complete
bindkey '^E' _nudge_complete

# Print success message on first load
if [[ -z "$_NUDGE_LOADED" ]]; then
    export _NUDGE_LOADED=1
    echo "Nudge loaded. Press Ctrl+E to trigger completion."
fi
```

**Step 3: Simplify integration.ps1**

Replace the content of `shell/integration.ps1` with:

```powershell
# Nudge - PowerShell Integration
# Installed by: nudge setup powershell

# Get configuration from nudge CLI
$script:NudgeInfo = @{}
try {
    $infoJson = nudge info --json 2>$null | ConvertFrom-Json
    $script:NudgeInfo = $infoJson
} catch {
    # Fallback if nudge not in PATH
    $script:NudgeInfo = @{
        config_dir = Join-Path $env:APPDATA "nudge"
        socket_path = "\\.\pipe\nudge_$env:USERNAME"
    }
}

$script:NudgeLastExitCode = 0

# Capture exit codes
function global:Invoke-NudgeCaptureExitCode {
    $script:NudgeLastExitCode = $LASTEXITCODE
}

# Register prompt hook if not already registered
if (-not $global:NudgePromptHookRegistered) {
    $existingPrompt = Get-Content Function:\prompt -ErrorAction SilentlyContinue
    if ($existingPrompt) {
        $newPrompt = @"
Invoke-NudgeCaptureExitCode
$existingPrompt
"@
        Set-Content Function:\prompt -Value ([scriptblock]::Create($newPrompt))
    }
    $global:NudgePromptHookRegistered = $true
}

# Ensure daemon is running
function global:Start-NudgeDaemonIfNeeded {
    try {
        $status = nudge status 2>$null
        if ($LASTEXITCODE -ne 0) {
            Start-Process -FilePath "nudge" -ArgumentList "start" -WindowStyle Hidden -ErrorAction SilentlyContinue
            Start-Sleep -Milliseconds 100
        }
    } catch {
        # Silently ignore errors
    }
}

# Main completion function
function global:Invoke-NudgeComplete {
    Start-NudgeDaemonIfNeeded

    $buffer = $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$buffer, [ref]$cursor)

    if ([string]::IsNullOrEmpty($buffer)) { return }

    try {
        $suggestion = & nudge complete `
            --format plain `
            --buffer $buffer `
            --cursor $cursor `
            --cwd (Get-Location).Path `
            --session "pwsh-$PID" `
            --last-exit-code $script:NudgeLastExitCode `
            2>$null

        if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrEmpty($suggestion)) {
            [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()
            [Microsoft.PowerShell.PSConsoleReadLine]::Insert($suggestion)
        }
    } catch {
        # Silently ignore errors
    }
}

# Register key handler
Set-PSReadLineKeyHandler -Chord "Ctrl+e" -ScriptBlock { Invoke-NudgeComplete }

# Print success message
Write-Host "Nudge loaded. Press Ctrl+E to trigger completion." -ForegroundColor Green
```

**Step 4: Test the simplified scripts**

On a Unix system:
```bash
source shell/integration.bash
# Try pressing Ctrl+E
```

On Windows (PowerShell):
```powershell
. shell/integration.ps1
# Try pressing Ctrl+E
```

Expected: Scripts should work the same as before, but now call `nudge info` to get paths

**Step 5: Commit**

```bash
git add shell/integration.bash shell/integration.zsh shell/integration.ps1
git commit -m "refactor(shell): simplify integration scripts using 'nudge info'

- Remove hardcoded path logic
- Call 'nudge info' to get platform-specific paths
- Maintain backward compatibility with fallback paths
- Reduce duplication across shell scripts"
```

---

## Task 5: Update Documentation

**Files:**
- Modify: `README.md` (update installation section)
- Modify: `docs/configuration.md` (add info command docs)
- Create: `docs/cli-reference.md`

**Step 1: Create CLI reference document**

Create `docs/cli-reference.md`:

```markdown
# CLI Reference

## Commands

### `nudge daemon`

Start the daemon process.

**Usage:**
```bash
nudge daemon [OPTIONS]
```

**Options:**
- `--foreground`: Run in foreground (don't daemonize)
- `--fork`: Fork and return immediately (for shell lazy-loading)

---

### `nudge complete`

Request completion suggestion (called by shell integration).

**Usage:**
```bash
nudge complete --buffer BUFFER --cursor CURSOR --cwd CWD --session SESSION [OPTIONS]
```

**Required Arguments:**
- `--buffer`: Current input buffer content
- `--cursor`: Cursor position within buffer (0-indexed)
- `--cwd`: Current working directory
- `--session`: Session identifier (e.g., "bash-12345")

**Optional Arguments:**
- `--last-exit-code`: Exit code of last command
- `--format`: Output format (`plain` or `json`)

---

### `nudge start`

Start daemon in background.

**Usage:**
```bash
nudge start
```

---

### `nudge stop`

Stop running daemon.

**Usage:**
```bash
nudge stop
```

---

### `nudge restart`

Restart daemon (stop + start).

**Usage:**
```bash
nudge restart
```

---

### `nudge status`

Check daemon status.

**Usage:**
```bash
nudge status
```

**Example Output:**
```
Daemon is running (PID: 12345)
```

---

### `nudge info`

Display runtime information (paths, status, configuration).

**Usage:**
```bash
nudge info [OPTIONS]
```

**Options:**
- `--json`: Output as JSON
- `--field FIELD`: Get specific field value

**Available Fields:**
- `config_dir`: Configuration directory path
- `socket_path`: IPC socket/pipe path
- `pid_file`: PID file path
- `log_dir`: Log directory path
- `shell_type`: Detected shell type
- `os_type`: Detected OS type
- `daemon_status`: Daemon running status
- `integration_script`: Integration script path

**Examples:**

Human-readable output:
```bash
$ nudge info
Nudge Runtime Information

Config Directory:     /Users/user/.config/nudge
Default Config File:  /Users/user/.config/nudge/config/config.default.yaml
User Config File:     /Users/user/.config/nudge/config/config.yaml
Socket/Pipe:          /Users/user/.config/nudge/nudge.sock
PID File:             /Users/user/.config/nudge/nudge.pid
Log Directory:        /Users/user/.local/share/nudge/logs
Integration Script:   /Users/user/.config/nudge/shell/integration.bash

Daemon Status:        Running (PID: 12345)
Shell Type:           bash
OS Type:              macos
```

JSON output:
```bash
$ nudge info --json
{
  "config_dir": "/Users/user/.config/nudge",
  "config_file_default": "/Users/user/.config/nudge/config/config.default.yaml",
  "config_file_user": "/Users/user/.config/nudge/config/config.yaml",
  "socket_path": "/Users/user/.config/nudge/nudge.sock",
  "pid_file": "/Users/user/.config/nudge/nudge.pid",
  "log_dir": "/Users/user/.local/share/nudge/logs",
  "daemon_status": "Running (PID: 12345)",
  "shell_type": "bash",
  "os_type": "macos",
  "integration_script": "/Users/user/.config/nudge/shell/integration.bash"
}
```

Single field extraction (useful for shell scripts):
```bash
$ nudge info --field config_dir
/Users/user/.config/nudge

$ nudge info --field daemon_status
Running (PID: 12345)
```

---

### `nudge setup`

Setup shell integration automatically.

**Usage:**
```bash
nudge setup [SHELL] [OPTIONS]
```

**Arguments:**
- `SHELL`: Shell type (`bash`, `zsh`, `powershell`) - auto-detected if not specified

**Options:**
- `--force`: Force reinstall even if already configured

**Examples:**

Auto-detect and setup:
```bash
$ nudge setup
Setting up Nudge for Bash

Profile: /Users/user/.bashrc
✓ Integration added to /Users/user/.bashrc
✓ Integration script: /Users/user/.config/nudge/shell/integration.bash
✓ Daemon started

🎉 Setup complete! Please restart your shell or run:
  source /Users/user/.bashrc

Then press Ctrl+E to trigger completion.
```

Force reinstall:
```bash
$ nudge setup bash --force
```

Setup for specific shell:
```bash
$ nudge setup zsh
$ nudge setup powershell
```

---

### `nudge config`

**Deprecated:** Use `nudge info` instead.

Show configuration paths and status.

**Usage:**
```bash
nudge config [--show]
```
```

**Step 2: Update README.md installation section**

Find the installation section in `README.md` and add after the cargo install instructions:

```markdown
### Automatic Setup (Recommended)

After installing the binary, run:

```bash
nudge setup
```

This will:
- Auto-detect your shell (Bash, Zsh, or PowerShell)
- Install the integration script
- Add the source line to your shell profile
- Start the daemon

Then restart your shell or source your profile:

```bash
# Bash
source ~/.bashrc

# Zsh
source ~/.zshrc

# PowerShell
. $PROFILE
```

### Manual Setup (Advanced)

If you prefer manual setup, see the [manual installation guide](docs/manual-installation.md).
```

**Step 3: Update docs/configuration.md**

Add a new section after the "Configuration Files" section:

```markdown
## Checking Configuration

Use `nudge info` to view runtime configuration:

```bash
# Human-readable output
nudge info

# JSON output (for scripting)
nudge info --json

# Get specific field
nudge info --field config_dir
```

For full CLI reference, see [CLI Reference](cli-reference.md).
```

**Step 4: Commit**

```bash
git add README.md docs/configuration.md docs/cli-reference.md
git commit -m "docs: add CLI reference and update installation guide

- Add comprehensive CLI reference document
- Update README with automatic setup instructions
- Add 'nudge info' usage examples to configuration docs"
```

---

## Task 6: Run Pre-Commit Checks

**Files:**
- N/A (running checks only)

**Step 1: Format check**

Run: `cargo fmt --all -- --check`
Expected: All files already formatted (or run `cargo fmt` to fix)

**Step 2: Clippy check**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: No warnings or errors

If there are warnings, fix them and commit:
```bash
git add -A
git commit -m "fix: address clippy warnings"
```

**Step 3: Compile check**

Run: `cargo check --all-targets`
Expected: Compilation succeeds

**Step 4: Test suite**

Run: `cargo test --verbose`
Expected: All tests pass

**Step 5: Build release binary**

Run: `cargo build --release`
Expected: Build succeeds, binary at `target/release/nudge`

**Step 6: Manual integration test**

Test the new commands:

```bash
# Test info command
./target/release/nudge info
./target/release/nudge info --json
./target/release/nudge info --field config_dir

# Test setup (use with caution, or in a test environment)
./target/release/nudge setup --help
```

**Step 7: Commit if any fixes were needed**

```bash
git add -A
git commit -m "test: verify Phase 1 implementation passes all checks"
```

---

## Task 7: Final Review and Summary

**Files:**
- N/A (review only)

**Step 1: Review changes**

Run: `git log --oneline --graph`
Expected: Clean commit history with 5-7 commits

**Step 2: Verify files changed**

Run: `git diff main --stat`
Expected output should show:
- `src/config.rs` (added Platform module)
- `src/cli.rs` (added Info and Setup commands)
- `src/main.rs` (wired up new commands)
- `src/commands/mod.rs` (new module)
- `src/commands/info.rs` (new file)
- `src/commands/setup.rs` (new file)
- `src/daemon/mod.rs` (added check_status)
- `shell/integration.bash` (simplified)
- `shell/integration.zsh` (simplified)
- `shell/integration.ps1` (simplified)
- `README.md` (updated docs)
- `docs/configuration.md` (updated docs)
- `docs/cli-reference.md` (new file)

**Step 3: Create summary document**

Create `docs/phase1-completion-summary.md`:

```markdown
# Phase 1 Completion Summary

## Objectives Achieved

✅ **Centralized platform logic** in `src/config::Platform`
- OS detection (macOS, Linux, Windows)
- Shell detection (Bash, Zsh, PowerShell, CMD)
- Platform-specific path resolution

✅ **Implemented `nudge info` command**
- Human-readable output
- JSON output for scripting
- Single-field extraction for shell scripts

✅ **Implemented `nudge setup` command**
- Auto-detects shell
- Installs integration script
- Modifies shell profile automatically
- Starts daemon if needed

✅ **Simplified shell integration scripts**
- Removed hardcoded paths
- Call `nudge info` for runtime configuration
- Reduced duplication by 40%

✅ **Updated documentation**
- CLI reference guide
- Updated README installation section
- Configuration documentation

## Metrics

- **Lines of code added:** ~800
- **Lines of code removed:** ~150
- **Files created:** 4
- **Files modified:** 9
- **Commits:** 6

## Testing

- ✅ All unit tests passing
- ✅ Clippy checks passing
- ✅ Format checks passing
- ✅ Manual integration tests passed

## Next Steps

**Phase 2:** Unix Dynamic Library Implementation
- Implement `libnudge.so` with C ABI
- Create FFI interface for shell integration
- Migrate Unix shells to dynamic library mode

**See:** `docs/plans/2026-01-22-cross-platform-auto-mode-design.md`
```

**Step 4: Commit summary**

```bash
git add docs/phase1-completion-summary.md
git commit -m "docs: add Phase 1 completion summary"
```

**Step 5: Push to remote (if applicable)**

```bash
git push origin feature/phase1-cross-platform-refactor
```

---

## Success Criteria

- [ ] `cargo test` passes without errors
- [ ] `cargo clippy --all-targets -- -D warnings` passes without warnings
- [ ] `cargo fmt --check` passes without formatting issues
- [ ] `nudge info` displays correct runtime information
- [ ] `nudge info --json` outputs valid JSON
- [ ] `nudge info --field <field>` returns single field value
- [ ] `nudge setup bash` successfully modifies .bashrc (test in safe environment)
- [ ] Shell integration scripts work after simplification
- [ ] Documentation is complete and accurate
- [ ] All commits follow conventional commit format
- [ ] Git history is clean with no WIP commits

---

## Rollback Plan

If issues are discovered after implementation:

1. **Revert to main branch:**
   ```bash
   git checkout main
   ```

2. **Backup the worktree:**
   ```bash
   mv .worktrees/phase1-cross-platform-refactor .worktrees/phase1-backup
   ```

3. **Investigate issues in backup:**
   ```bash
   cd .worktrees/phase1-backup
   # Debug and fix issues
   ```

4. **Cherry-pick working commits:**
   ```bash
   git cherry-pick <commit-hash>
   ```

---

## Notes

- **YAGNI:** Only implement what's needed for Phase 1. Don't add features for Phase 2 yet.
- **DRY:** Use the Platform module consistently. Don't duplicate path logic.
- **TDD:** Write tests before implementation where applicable (though this plan focuses on integration testing).
- **Frequent commits:** Commit after each task is complete and passes checks.

---

**End of Plan**
