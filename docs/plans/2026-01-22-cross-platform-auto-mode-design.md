# Cross-Platform Architecture and Auto Mode Design

**Date**: 2026-01-22
**Status**: Design Approved
**Authors**: User + Claude

## Executive Summary

This design addresses two critical issues in nudge:

1. **Cross-platform implementation complexity**: Configuration paths, shell integration scripts, and installation logic are scattered across multiple files with significant duplication.
2. **Non-functional auto mode**: The `trigger.mode` configuration exists but is not implemented.

**Solution**: Hybrid platform-specific approach
- **Unix (Linux/macOS)**: Dynamic library (`.so`/`.dylib`) with native shell hooks for optimal performance and true auto mode
- **Windows (PowerShell/CMD)**: Enhanced CLI + daemon architecture with PSReadLine integration

## Current Problems

### Problem 1: Cross-Platform Architecture Issues

**Configuration path fragmentation**:
- Platform-specific path logic duplicated across `src/config.rs`, shell scripts, and installation scripts
- macOS uses `~/Library/Application Support/`, Linux uses `~/.config/`, Windows uses `%APPDATA%`
- Changes require syncing multiple files

**Shell integration duplication**:
- `integration.bash`, `integration.zsh`, `integration.ps1`, `integration.cmd` all implement similar logic:
  - Daemon liveness check
  - IPC communication
  - Exit code capture
  - Hotkey binding
- Approximately 70% code overlap with platform-specific syntax

**Complex installation**:
- Users must manually edit shell profiles
- Need to understand platform-specific paths
- No automated setup command

**Compile-time platform detection**:
- Heavy use of `#[cfg(windows)]` and `#[cfg(unix)]`
- Lacks runtime adaptation for cross-platform differences

### Problem 2: Auto Mode Not Implemented

**Configuration defined but unused**:
```yaml
trigger:
  mode: auto              # Defined in config.rs
  auto_delay_ms: 500      # Never referenced in code
```

**Search results**:
- `grep -r "config.trigger"` → No matches
- Configuration struct exists, but no code consumes it

**User expectation**:
- Input debounce: Trigger completion after 500ms of idle input
- Inline preview: Display suggestions as gray text (like GitHub Copilot / Fish shell)
- Accept key: Tab or Right Arrow to accept suggestion

## Design Solution

### Architecture Overview

**Core Philosophy**: Use platform-optimal solutions rather than forcing uniformity.

#### Unix (Linux/macOS) - Dynamic Library Approach

```
┌─────────────────────────────────────────────────┐
│  Shell (Bash/Zsh)                               │
│  ┌───────────────────────────────────┐          │
│  │ integration.bash / integration.zsh│          │
│  │ - Load libnudge.so via dlopen     │          │
│  │ - Bind hotkey → nudge_complete()  │          │
│  │ - Hook input events (auto mode)   │          │
│  └───────────────┬───────────────────┘          │
└──────────────────┼─────────────────────────────┘
                   │ FFI (C ABI)
                   ▼
┌─────────────────────────────────────────────────┐
│  libnudge.so (Rust compiled to dynamic library) │
│  ┌─────────────────────────────────────────┐    │
│  │ • LLM client (async with Tokio)         │    │
│  │ • Context collector                     │    │
│  │ • Sanitizer                             │    │
│  │ • Cache (buffer hash → suggestion)      │    │
│  │ • Auto mode: background thread +        │    │
│  │   debounce + callback                   │    │
│  └─────────────────────────────────────────┘    │
└─────────────────────────────────────────────────┘
```

**Benefits**:
- No IPC overhead (direct function calls)
- Native shell hook integration
- True auto mode with low latency
- Easier to implement inline preview

#### Windows (PowerShell/CMD) - Enhanced CLI Approach

```
┌─────────────────────────────────────────────────┐
│  PowerShell                                     │
│  ┌───────────────────────────────────┐          │
│  │ integration.ps1                   │          │
│  │ - Ctrl+E → nudge complete CLI     │          │
│  │ - PSReadLine predictor (auto)     │          │
│  └───────────────┬───────────────────┘          │
└──────────────────┼─────────────────────────────┘
                   │ CLI invocation
                   ▼
┌─────────────────────────────────────────────────┐
│  nudge.exe (Enhanced CLI)                       │
│  • nudge info --json                            │
│  • nudge setup powershell                       │
│  • nudge complete (existing)                    │
└───────────────┬─────────────────────────────────┘
                │ IPC (Named Pipe)
                ▼
┌─────────────────────────────────────────────────┐
│  nudge daemon (existing architecture)           │
└─────────────────────────────────────────────────┘
```

**Benefits**:
- Maintains stable daemon + IPC architecture
- Simpler installation with `nudge setup`
- Leverages PowerShell 7.2+ PSReadLine API for auto mode
- Fallback to CLI mode is trivial (already implemented)

### Detailed Component Design

#### 1. Centralized Platform Logic (`src/config.rs`)

```rust
pub struct Platform {
    pub os: OsType,
    pub shell: ShellType,
}

pub enum OsType {
    Linux,
    MacOS,
    Windows,
}

pub enum ShellType {
    Bash,
    Zsh,
    PowerShell,
    Cmd,
}

impl Platform {
    /// Detect platform at runtime
    pub fn detect() -> Self {
        let os = if cfg!(target_os = "macos") {
            OsType::MacOS
        } else if cfg!(target_os = "linux") {
            OsType::Linux
        } else if cfg!(target_os = "windows") {
            OsType::Windows
        } else {
            panic!("Unsupported OS");
        };

        let shell = Self::detect_shell();

        Self { os, shell }
    }

    fn detect_shell() -> ShellType {
        // Check $SHELL, $PSModulePath, etc.
        // ...
    }

    pub fn config_dir(&self) -> PathBuf {
        match self.os {
            OsType::MacOS => PathBuf::from(env::var("HOME").unwrap())
                .join("Library/Application Support/nudge"),
            OsType::Linux => {
                let base = env::var("XDG_CONFIG_HOME")
                    .unwrap_or_else(|_| format!("{}/.config", env::var("HOME").unwrap()));
                PathBuf::from(base).join("nudge")
            },
            OsType::Windows => PathBuf::from(env::var("APPDATA").unwrap())
                .join("nudge"),
        }
    }

    pub fn integration_script(&self) -> PathBuf {
        let filename = match self.shell {
            ShellType::Bash => "integration.bash",
            ShellType::Zsh => "integration.zsh",
            ShellType::PowerShell => "integration.ps1",
            ShellType::Cmd => "integration.cmd",
        };
        self.config_dir().join("shell").join(filename)
    }

    pub fn lib_path(&self) -> Option<PathBuf> {
        match self.os {
            OsType::MacOS => Some(self.config_dir().join("lib/libnudge.dylib")),
            OsType::Linux => Some(self.config_dir().join("lib/libnudge.so")),
            OsType::Windows => None, // Windows uses CLI mode
        }
    }

    pub fn socket_path(&self) -> PathBuf {
        match self.os {
            OsType::Windows => {
                let username = env::var("USERNAME").unwrap_or_else(|_| "default".into());
                PathBuf::from(format!(r"\\.\pipe\nudge_{}", username))
            },
            _ => self.config_dir().join("nudge.sock"),
        }
    }
}
```

#### 2. Unix Dynamic Library - C ABI Interface

**File**: `src/ffi/mod.rs`

```rust
use std::ffi::{CStr, CString, c_char, c_int, c_uint, c_void};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

pub struct NudgeContext {
    config: Config,
    llm_client: LlmClient,
    runtime: Runtime,
    cache: Arc<Mutex<HashMap<u64, String>>>, // buffer hash -> suggestion
}

/// Initialize nudge context (call once on shell startup)
///
/// # Safety
/// `config_path` must be a valid null-terminated C string or NULL
#[no_mangle]
pub unsafe extern "C" fn nudge_init(config_path: *const c_char) -> *mut NudgeContext {
    let config = if config_path.is_null() {
        Config::load().unwrap_or_default()
    } else {
        let path = CStr::from_ptr(config_path).to_str().unwrap();
        Config::load_from_path(&PathBuf::from(path)).unwrap_or_default()
    };

    let runtime = Runtime::new().expect("Failed to create Tokio runtime");
    let llm_client = LlmClient::new(&config.model);

    let ctx = NudgeContext {
        config,
        llm_client,
        runtime,
        cache: Arc::new(Mutex::new(HashMap::new())),
    };

    Box::into_raw(Box::new(ctx))
}

/// Manual mode: Trigger completion on hotkey press
///
/// # Safety
/// - `ctx` must be a valid pointer from `nudge_init()`
/// - `buffer` and `cwd` must be valid null-terminated C strings
/// - `callback` will be called with the suggestion (or NULL on error)
#[no_mangle]
pub unsafe extern "C" fn nudge_complete(
    ctx: *mut NudgeContext,
    buffer: *const c_char,
    cursor: c_int,
    cwd: *const c_char,
    session_id: *const c_char,
    callback: extern "C" fn(*const c_char, *mut c_void),
    user_data: *mut c_void,
) -> c_int {
    let ctx = &mut *ctx;
    let buffer_str = CStr::from_ptr(buffer).to_str().unwrap();
    let cwd_str = CStr::from_ptr(cwd).to_str().unwrap();
    let session_str = CStr::from_ptr(session_id).to_str().unwrap();

    // Check cache
    let hash = calculate_hash(buffer_str, cwd_str);
    if let Some(cached) = ctx.cache.lock().unwrap().get(&hash) {
        let c_str = CString::new(cached.clone()).unwrap();
        callback(c_str.as_ptr(), user_data);
        return 0;
    }

    // Async LLM request
    ctx.runtime.spawn(async move {
        // Build context, call LLM, sanitize, etc.
        match get_completion(buffer_str, cursor, cwd_str, session_str).await {
            Ok(suggestion) => {
                // Cache result
                ctx.cache.lock().unwrap().insert(hash, suggestion.clone());

                let c_str = CString::new(suggestion).unwrap();
                callback(c_str.as_ptr(), user_data);
            }
            Err(_) => {
                callback(std::ptr::null(), user_data);
            }
        }
    });

    0 // Success
}

/// Auto mode: Start background input monitoring
///
/// # Safety
/// `ctx` must be a valid pointer from `nudge_init()`
#[no_mangle]
pub unsafe extern "C" fn nudge_auto_start(
    ctx: *mut NudgeContext,
    delay_ms: c_uint,
    callback: extern "C" fn(*const c_char, *mut c_void),
    user_data: *mut c_void,
) -> c_int {
    let ctx = &mut *ctx;

    // Start background thread for debounce logic
    // Hook into readline/zle events
    // On idle timeout, call nudge_complete()

    // Implementation details in Phase 3
    0
}

/// Free context resources
///
/// # Safety
/// `ctx` must be a valid pointer from `nudge_init()`, and should not be used after this call
#[no_mangle]
pub unsafe extern "C" fn nudge_free(ctx: *mut NudgeContext) {
    if !ctx.is_null() {
        let _ = Box::from_raw(ctx);
    }
}
```

#### 3. Shell Integration Scripts (Simplified)

**File**: `shell/integration.bash` (new version)

```bash
#!/usr/bin/env bash
# Nudge - Bash Integration (Dynamic Library Version)

# Get paths from nudge CLI
NUDGE_CONFIG_DIR=$(nudge info --field config_dir 2>/dev/null)
NUDGE_LIB_PATH=$(nudge info --field lib_path 2>/dev/null)
NUDGE_MODE=$(nudge info --field trigger_mode 2>/dev/null)

# Load dynamic library if available
_nudge_load_library() {
    if [[ -z "$NUDGE_LIB_PATH" ]] || [[ ! -f "$NUDGE_LIB_PATH" ]]; then
        return 1
    fi

    # Use a C shim to load .so and expose functions to bash
    # (Bash doesn't have native FFI, need small C wrapper)
    if [[ -f "$NUDGE_CONFIG_DIR/bin/nudge-ffi" ]]; then
        source <(nudge-ffi bash-bindings)
        return 0
    fi

    return 1
}

# Manual mode completion
_nudge_complete_manual() {
    local suggestion
    suggestion=$(nudge-ffi complete "$READLINE_LINE" "$READLINE_POINT" "$PWD" "bash-$$")

    if [[ -n "$suggestion" ]]; then
        READLINE_LINE="$suggestion"
        READLINE_POINT=${#READLINE_LINE}
    fi
}

# Fallback to CLI mode
_nudge_complete_cli() {
    _nudge_ensure_daemon  # Existing logic
    local suggestion
    suggestion=$(nudge complete --format plain \
        --buffer "$READLINE_LINE" \
        --cursor "$READLINE_POINT" \
        --cwd "$PWD" \
        --session "bash-$$" 2>/dev/null)

    if [[ -n "$suggestion" ]]; then
        READLINE_LINE="$suggestion"
        READLINE_POINT=${#READLINE_LINE}
    fi
}

# Try dynamic library, fallback to CLI
if _nudge_load_library; then
    echo "Nudge loaded (dynamic library mode)"
    bind -x '"\C-e": _nudge_complete_manual'

    # Auto mode (if enabled)
    if [[ "$NUDGE_MODE" == "auto" ]]; then
        nudge-ffi auto-start 500
    fi
else
    echo "Nudge loaded (CLI mode)"
    bind -x '"\C-e": _nudge_complete_cli'
fi
```

**File**: `shell/integration.ps1` (new version)

```powershell
# Nudge - PowerShell Integration (Enhanced CLI Version)

# Get configuration from nudge CLI
$script:NudgeInfo = nudge info --json 2>$null | ConvertFrom-Json
$script:NudgeLastExitCode = 0

# Capture exit codes
$script:NudgePromptHook = {
    $script:NudgeLastExitCode = $LASTEXITCODE
}

if (-not $global:NudgePromptHookRegistered) {
    $existingPrompt = Get-Content Function:\prompt -ErrorAction SilentlyContinue
    if ($existingPrompt) {
        $newPrompt = @"
`$script:NudgeLastExitCode = `$LASTEXITCODE
$existingPrompt
"@
        Set-Content Function:\prompt -Value $newPrompt
    }
    $global:NudgePromptHookRegistered = $true
}

# Ensure daemon is running
function Start-NudgeDaemon {
    $status = nudge status 2>$null
    if ($LASTEXITCODE -ne 0) {
        Start-Process -FilePath "nudge" -ArgumentList "start" -WindowStyle Hidden -ErrorAction SilentlyContinue
        Start-Sleep -Milliseconds 100
    }
}

# Manual mode completion
function Invoke-NudgeComplete {
    Start-NudgeDaemon

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
        # Silent failure
    }
}

# Register hotkey
$hotkey = if ($script:NudgeInfo.trigger_hotkey) { $script:NudgeInfo.trigger_hotkey } else { "Ctrl+e" }
Set-PSReadLineKeyHandler -Chord $hotkey -ScriptBlock { Invoke-NudgeComplete }

# Auto mode (PowerShell 7.2+ only)
if ($PSVersionTable.PSVersion.Major -ge 7 -and $PSVersionTable.PSVersion.Minor -ge 2) {
    if ($script:NudgeInfo.trigger_mode -eq "auto") {
        # Use PSReadLine predictor API
        Set-PSReadLineOption -PredictionSource HistoryAndPlugin
        Set-PSReadLineKeyHandler -Key Tab -Function AcceptSuggestion

        # Register custom predictor (implementation in Phase 4)
        # Import-Module NudgePredictor
    }
}

Write-Host "Nudge loaded. Press $hotkey to trigger completion." -ForegroundColor Green
```

#### 4. New CLI Commands

**File**: `src/cli.rs`

```rust
#[derive(Parser)]
#[command(name = "nudge")]
pub enum Cli {
    // ... existing commands (daemon, complete, start, stop, status)

    /// Display runtime information (paths, status, config)
    Info {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Get specific field (config_dir, lib_path, socket_path, trigger_mode, etc.)
        #[arg(long)]
        field: Option<String>,
    },

    /// Setup shell integration automatically
    Setup {
        /// Shell type (bash, zsh, powershell, cmd) - auto-detect if not specified
        shell: Option<String>,

        /// Force reinstall even if already configured
        #[arg(long)]
        force: bool,
    },
}
```

**File**: `src/commands/info.rs`

```rust
pub fn run_info(json: bool, field: Option<String>) -> Result<()> {
    let platform = Platform::detect();
    let config = Config::load()?;

    let info = InfoOutput {
        config_dir: platform.config_dir().display().to_string(),
        config_file: Config::default_config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
        socket_path: Config::socket_path().display().to_string(),
        pid_file: Config::pid_path().display().to_string(),
        lib_path: platform.lib_path().map(|p| p.display().to_string()),
        daemon_status: check_daemon_status()?,
        trigger_mode: format!("{:?}", config.trigger.mode).to_lowercase(),
        trigger_hotkey: config.trigger.hotkey.clone(),
        shell_type: format!("{:?}", platform.shell).to_lowercase(),
        os_type: format!("{:?}", platform.os).to_lowercase(),
    };

    if let Some(field_name) = field {
        // Output single field for shell scripts
        let value = match field_name.as_str() {
            "config_dir" => info.config_dir,
            "lib_path" => info.lib_path.unwrap_or_default(),
            "trigger_mode" => info.trigger_mode,
            _ => anyhow::bail!("Unknown field: {}", field_name),
        };
        println!("{}", value);
    } else if json {
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        // Human-readable format
        println!("Config Dir:    {}", info.config_dir);
        println!("Config File:   {}", info.config_file);
        println!("Socket/Pipe:   {}", info.socket_path);
        println!("PID File:      {}", info.pid_file);
        if let Some(lib) = info.lib_path {
            println!("Library:       {}", lib);
        }
        println!("Daemon Status: {}", info.daemon_status);
        println!("Trigger Mode:  {}", info.trigger_mode);
        println!("Shell Type:    {}", info.shell_type);
        println!("OS Type:       {}", info.os_type);
    }

    Ok(())
}

fn check_daemon_status() -> Result<String> {
    let pid_path = Config::pid_path();
    if !pid_path.exists() {
        return Ok("Not running".to_string());
    }

    let pid: u32 = std::fs::read_to_string(&pid_path)?.trim().parse()?;

    // Check if process exists
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        match kill(Pid::from_raw(pid as i32), Signal::SIGCONT) {
            Ok(_) => Ok(format!("Running (PID: {})", pid)),
            Err(_) => Ok("Not running (stale PID)".to_string()),
        }
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION};
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
            if handle != 0 {
                Ok(format!("Running (PID: {})", pid))
            } else {
                Ok("Not running (stale PID)".to_string())
            }
        }
    }
}
```

**File**: `src/commands/setup.rs`

```rust
pub fn run_setup(shell: Option<String>, force: bool) -> Result<()> {
    let platform = Platform::detect();
    let detected_shell = shell.as_deref().unwrap_or_else(|| {
        match platform.shell {
            ShellType::Bash => "bash",
            ShellType::Zsh => "zsh",
            ShellType::PowerShell => "powershell",
            ShellType::Cmd => "cmd",
        }
    });

    match detected_shell {
        "bash" => setup_bash(&platform, force),
        "zsh" => setup_zsh(&platform, force),
        "powershell" => setup_powershell(&platform, force),
        "cmd" => setup_cmd(&platform, force),
        _ => anyhow::bail!("Unsupported shell: {}", detected_shell),
    }
}

fn setup_bash(platform: &Platform, force: bool) -> Result<()> {
    let bashrc = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".bashrc");

    let integration_script = platform.integration_script();
    let source_line = format!("source \"{}\"", integration_script.display());

    // Check if already configured
    if bashrc.exists() && !force {
        let content = std::fs::read_to_string(&bashrc)?;
        if content.contains(&source_line) {
            println!("✓ Nudge is already configured in {}", bashrc.display());
            return Ok(());
        }
    }

    // Copy integration script to config dir
    let script_content = include_str!("../../shell/integration.bash");
    std::fs::create_dir_all(integration_script.parent().unwrap())?;
    std::fs::write(&integration_script, script_content)?;

    // Append to .bashrc
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&bashrc)?;

    use std::io::Write;
    writeln!(file, "\n# Nudge - LLM-powered CLI completion")?;
    writeln!(file, "{}", source_line)?;

    println!("✓ Nudge integration added to {}", bashrc.display());
    println!("✓ Integration script: {}", integration_script.display());
    println!("\nPlease restart your shell or run:");
    println!("  source {}", bashrc.display());

    // Start daemon if not running
    if !daemon_is_running()? {
        println!("\nStarting daemon...");
        crate::commands::start::run_start()?;
        println!("✓ Daemon started");
    }

    Ok(())
}

fn setup_powershell(platform: &Platform, force: bool) -> Result<()> {
    // Detect PowerShell profile path
    let profile_path = if let Ok(profile) = std::env::var("PROFILE") {
        PathBuf::from(profile)
    } else {
        // Fallback detection
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1")
    };

    let integration_script = platform.integration_script();
    let source_line = format!(". \"{}\"", integration_script.display());

    // Check if already configured
    if profile_path.exists() && !force {
        let content = std::fs::read_to_string(&profile_path)?;
        if content.contains(&source_line) {
            println!("✓ Nudge is already configured in {}", profile_path.display());
            return Ok(());
        }
    }

    // Copy integration script
    let script_content = include_str!("../../shell/integration.ps1");
    std::fs::create_dir_all(integration_script.parent().unwrap())?;
    std::fs::write(&integration_script, script_content)?;

    // Append to profile
    std::fs::create_dir_all(profile_path.parent().unwrap())?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&profile_path)?;

    use std::io::Write;
    writeln!(file, "\n# Nudge - LLM-powered CLI completion")?;
    writeln!(file, "{}", source_line)?;

    println!("✓ Nudge integration added to {}", profile_path.display());
    println!("\nPlease restart PowerShell or run:");
    println!("  . $PROFILE");

    // Start daemon
    if !daemon_is_running()? {
        println!("\nStarting daemon...");
        crate::commands::start::run_start()?;
        println!("✓ Daemon started");
    }

    Ok(())
}

// Similar implementations for setup_zsh() and setup_cmd()
```

#### 5. Configuration Enhancements

**File**: `config/config.default.yaml.template`

```yaml
# Nudge Configuration (Default)
# This file is updated automatically on upgrades.
# User customizations should go in config.yaml

model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"
  api_key: null
  api_key_env: null
  timeout_ms: 5000

context:
  history_window: 20
  include_cwd_listing: true
  include_exit_code: true
  include_system_info: true
  similar_commands_enabled: true
  similar_commands_window: 200
  similar_commands_max: 5
  max_files_in_listing: 50
  max_total_tokens: 4000
  priorities:
    history: 80
    cwd_listing: 60
    plugins: 40

plugins:
  git:
    enabled: true
    depth: "standard"  # light | standard | detailed
    recent_commits: 5
    priority: 50
  plugin_dir: null

trigger:
  mode: "manual"              # manual | auto
  hotkey: "\C-e"             # Ctrl+E (bash/zsh) or "Ctrl+e" (PowerShell)
  auto_delay_ms: 500          # Delay before auto-trigger (auto mode only)
  auto_inline_preview: true   # Show inline preview (Unix only)
  auto_accept_key: "Tab"      # Key to accept suggestion (Tab | RightArrow)

privacy:
  sanitize_enabled: true
  custom_patterns: []
  block_dangerous: true
  custom_blocked: []

log:
  level: "info"               # trace | debug | info | warn | error
  file_enabled: false

# Optional: Override platform detection
platform:
  force_cli_mode: false       # Unix: force CLI mode instead of dynamic library
```

#### 6. Auto Mode Implementation Details

**Phase 3: Unix Auto Mode**

**Mechanism**:
- Hook into readline's `rl_pre_input_hook` (Bash) or ZLE's `zle-line-pre-redraw` (Zsh)
- On every input change, reset a 500ms debounce timer
- When timer expires, trigger LLM request asynchronously
- Display suggestion as gray text after cursor using ANSI escape codes

**Rust implementation** (`src/ffi/auto_mode.rs`):

```rust
static AUTO_TIMER: Lazy<Mutex<Option<JoinHandle<()>>>> = Lazy::new(|| Mutex::new(None));
static LAST_BUFFER: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

pub unsafe extern "C" fn nudge_auto_start(
    ctx: *mut NudgeContext,
    delay_ms: c_uint,
    callback: extern "C" fn(*const c_char, *mut c_void),
    user_data: *mut c_void,
) -> c_int {
    let ctx = &*ctx;
    let delay = Duration::from_millis(delay_ms as u64);

    // This function is called on every input change from the shell
    // Debounce logic: cancel previous timer, start new one

    let mut timer = AUTO_TIMER.lock().unwrap();
    if let Some(handle) = timer.take() {
        handle.abort(); // Cancel previous request
    }

    let ctx_ptr = ctx as *const NudgeContext;
    let handle = tokio::spawn(async move {
        tokio::time::sleep(delay).await;

        // Trigger completion
        let ctx = unsafe { &*ctx_ptr };
        let buffer = LAST_BUFFER.lock().unwrap().clone();

        if buffer.is_empty() || buffer.len() == 1 {
            return; // Don't trigger for empty or single-char input
        }

        match get_completion_async(ctx, &buffer).await {
            Ok(suggestion) => {
                let c_str = CString::new(suggestion).unwrap();
                callback(c_str.as_ptr(), user_data);
            }
            Err(_) => {
                callback(std::ptr::null(), user_data);
            }
        }
    });

    *timer = Some(handle);

    0
}

pub unsafe extern "C" fn nudge_auto_update_buffer(
    ctx: *mut NudgeContext,
    buffer: *const c_char,
) {
    let buffer_str = CStr::from_ptr(buffer).to_str().unwrap();
    *LAST_BUFFER.lock().unwrap() = buffer_str.to_string();
}
```

**Shell integration** (`integration.bash`):

```bash
# Auto mode: hook into readline
if [[ "$NUDGE_MODE" == "auto" ]]; then
    _nudge_auto_callback() {
        # Called by Rust library with suggestion
        local suggestion="$1"

        # Display as gray text after cursor
        # Use ANSI: \e[90m (gray) + suggestion + \e[0m (reset)
        printf '\e[90m%s\e[0m' "$suggestion"
    }

    # Register with dynamic library
    nudge-ffi auto-start 500 _nudge_auto_callback

    # Hook every input change
    _nudge_input_hook() {
        nudge-ffi auto-update-buffer "$READLINE_LINE"
    }
    bind -x '"\C-x\C-r": _nudge_input_hook'  # Called on every keystroke

    # Accept suggestion with Tab
    _nudge_accept_suggestion() {
        # Logic to accept the displayed gray text
        # (Implementation depends on how we store the suggestion)
    }
    bind -x '"\t": _nudge_accept_suggestion'
fi
```

**Phase 4: Windows Auto Mode**

**PowerShell 7.2+ implementation**:

```powershell
# Create custom predictor class
class NudgePredictor : System.Management.Automation.Subsystem.Prediction.ICommandPredictor {
    [System.Guid] $Id = [System.Guid]::NewGuid()
    [string] $Name = 'Nudge'
    [string] $Description = 'LLM-powered command prediction'

    [System.Collections.Generic.IEnumerable[System.Management.Automation.Subsystem.Prediction.PredictiveSuggestion]] GetSuggestion(
        [System.Management.Automation.Subsystem.Prediction.PredictionContext] $context
    ) {
        # Call nudge complete --format json
        $result = & nudge complete --format json `
            --buffer $context.InputAst.Extent.Text `
            --cursor $context.CursorPosition.Offset `
            --cwd (Get-Location).Path `
            --session "pwsh-$PID" `
            2>$null

        if ($result) {
            $data = $result | ConvertFrom-Json
            $suggestion = [System.Management.Automation.Subsystem.Prediction.PredictiveSuggestion]::new(
                $data.suggestion,
                $this
            )
            return @($suggestion)
        }

        return @()
    }
}

# Register predictor
[System.Management.Automation.Subsystem.SubsystemManager]::RegisterSubsystem(
    [System.Management.Automation.Subsystem.SubsystemKind]::CommandPredictor,
    [NudgePredictor]::new()
)

Set-PSReadLineOption -PredictionSource HistoryAndPlugin
Set-PSReadLineKeyHandler -Key Tab -Function AcceptSuggestion
```

### Error Handling and Fallback Strategy

#### Degradation Levels

1. **Level 1 (Best)**: Unix dynamic library + Auto mode
2. **Level 2**: Unix dynamic library + Manual mode
3. **Level 3**: CLI + Daemon + Manual mode (current implementation)
4. **Level 4 (Fallback)**: Direct LLM call without daemon

#### Error Handling Principles

**User-visible errors** (show clear message):
- Daemon startup failure → "Daemon failed to start. Check config with: nudge info"
- LLM API error → "LLM request failed. Verify endpoint/API key in config."
- Config parse error → "Config syntax error at line X: [error details]"

**Silent errors** (log but don't interrupt):
- Auto mode LLM timeout → Cancel request, no user notification
- Cache miss → Fetch from LLM transparently
- Git plugin timeout (>50ms) → Skip git context, continue with other context

**Logging levels**:
- `error`: Core failures (daemon crash, config load failure)
- `warn`: Degradation (library load failed → CLI fallback, plugin timeout)
- `info`: Normal operations (daemon start, config loaded)
- `debug`: Detailed tracing (LLM request/response, context building, cache hits)

#### Fallback Implementation

**Unix shell integration**:

```bash
# Try dynamic library first
if _nudge_load_library; then
    echo "Nudge loaded (dynamic library mode)"
    _NUDGE_MODE="library"
else
    echo "Warning: libnudge.so not found, using CLI mode"
    _NUDGE_MODE="cli"

    # Ensure daemon is running for CLI mode
    _nudge_ensure_daemon() {
        if [[ ! -S "$NUDGE_SOCKET" ]]; then
            nudge start 2>/dev/null
        fi
    }
fi

# Unified completion function
_nudge_complete() {
    if [[ "$_NUDGE_MODE" == "library" ]]; then
        nudge-ffi complete "$READLINE_LINE" "$READLINE_POINT" "$PWD" "bash-$$"
    else
        _nudge_ensure_daemon
        nudge complete --format plain --buffer "$READLINE_LINE" --cursor "$READLINE_POINT" \
            --cwd "$PWD" --session "bash-$$" 2>/dev/null
    fi
}
```

### Testing Strategy

#### Unit Tests

**File**: `tests/platform_detection.rs`

```rust
#[test]
fn test_platform_detection() {
    let platform = Platform::detect();

    #[cfg(target_os = "macos")]
    assert_eq!(platform.os, OsType::MacOS);

    #[cfg(target_os = "linux")]
    assert_eq!(platform.os, OsType::Linux);

    #[cfg(target_os = "windows")]
    assert_eq!(platform.os, OsType::Windows);
}

#[test]
fn test_config_dir_paths() {
    let platform = Platform::detect();
    let config_dir = platform.config_dir();

    assert!(config_dir.is_absolute());

    #[cfg(target_os = "macos")]
    assert!(config_dir.to_str().unwrap().contains("Library/Application Support"));

    #[cfg(target_os = "linux")]
    assert!(config_dir.to_str().unwrap().contains(".config") ||
            config_dir.to_str().unwrap().contains("XDG_CONFIG_HOME"));

    #[cfg(target_os = "windows")]
    assert!(config_dir.to_str().unwrap().contains("AppData"));
}
```

#### Integration Tests

**File**: `tests/cli_commands.rs`

```rust
#[test]
fn test_nudge_info_json() {
    let output = Command::new("cargo")
        .args(&["run", "--", "info", "--json"])
        .output()
        .expect("Failed to execute nudge info");

    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["config_dir"].is_string());
    assert!(json["trigger_mode"].is_string());
}

#[test]
fn test_nudge_info_field() {
    let output = Command::new("cargo")
        .args(&["run", "--", "info", "--field", "config_dir"])
        .output()
        .expect("Failed to execute nudge info");

    assert!(output.status.success());
    let path = String::from_utf8(output.stdout).unwrap().trim().to_string();
    assert!(!path.is_empty());
    assert!(PathBuf::from(&path).is_absolute());
}
```

#### Manual Testing Matrix

| Platform | Shell | Mode | Test Cases |
|----------|-------|------|------------|
| macOS | Bash | Library + Manual | Hotkey trigger, completion accuracy |
| macOS | Bash | Library + Auto | Debounce timing, inline preview, accept key |
| macOS | Zsh | Library + Manual | Same as Bash |
| macOS | Zsh | Library + Auto | Same as Bash |
| Linux | Bash | Library + Manual | Same as macOS |
| Linux | Bash | Library + Auto | Same as macOS |
| Linux | Zsh | Library + Manual | Same as macOS |
| Linux | Zsh | Library + Auto | Same as macOS |
| Windows | PowerShell 7.2+ | CLI + Auto (PSReadLine) | Predictor integration, Tab acceptance |
| Windows | PowerShell 5.1 | CLI + Manual | Basic hotkey trigger |
| Windows | CMD | CLI + Manual | Basic hotkey trigger |

**Regression tests**:
- Verify existing CLI + daemon mode still works on all platforms
- Test fallback from library to CLI when .so is missing
- Verify backward compatibility with old config files

### Implementation Roadmap

#### Phase 1: Foundation Refactoring (1-2 weeks) ✅ COMPLETED

**Goal**: Centralize platform logic, eliminate path duplication

**Tasks**:
- [x] Create `src/config/platform.rs` module with `Platform` struct
- [x] Implement runtime platform detection (`detect()`)
- [x] Implement path methods (`config_dir()`, `socket_path()`, etc.)
- [x] Migrate existing path logic from shell scripts to Rust
- [x] Implement `nudge info` command with JSON output
- [x] Implement `nudge setup` command for Bash/Zsh/PowerShell
- [x] Update shell integration scripts to call `nudge info`
- [x] Update installation scripts to use `nudge setup`
- [x] Write unit tests for platform detection and paths
- [x] Write integration tests for new CLI commands

**Success criteria**:
- ✅ Zero hardcoded paths in shell scripts
- ✅ `nudge setup bash` successfully configures .bashrc
- ✅ All existing tests pass (backward compatibility)

#### Phase 2: Unix Dynamic Library (2-3 weeks) ✅ COMPLETED

**Goal**: Implement libnudge.so with manual mode

**Tasks**:
- [x] Create `src/ffi/` module structure
- [x] Implement C ABI interface (`nudge_init`, `nudge_complete`, `nudge_free`)
- [x] Handle FFI safety (null checks, CString conversions, panic catching)
- [x] Create Tokio runtime for async LLM calls
- [x] Implement completion cache (buffer hash → suggestion)
- [x] Build dynamic library target in `Cargo.toml`
- [x] Create C header file (`include/nudge.h`)
- [x] Update `integration.bash` to load library with fallback
- [x] Update `integration.zsh` to load library with fallback
- [x] Test on macOS and Linux
- [x] Write FFI integration tests (12 tests)
- [x] Document library API in `docs/ffi-api.md`

**Success criteria**:
- ✅ Dynamic library builds correctly (libnudge.dylib/libnudge.so)
- ✅ All FFI symbols exported correctly
- ✅ Fallback to CLI mode works when library missing
- ✅ No panics cross FFI boundary

#### Phase 3: Unix Auto Mode (2 weeks) ✅ COMPLETED

**Goal**: Implement true auto mode with inline preview

**Tasks**:
- [x] Implement `nudge_auto_start` FFI function
- [x] Create background thread for debounce timer
- [x] Implement buffer change detection
- [x] Add request cancellation on new input
- [x] Implement callback mechanism to shell
- [x] Hook readline/ZLE input events in Bash/Zsh
- [x] Implement inline preview with ANSI escape codes
- [x] Implement suggestion acceptance (Tab/Right Arrow)
- [x] Add configuration validation for auto mode settings
- [x] Test debounce timing (verify 500ms delay)
- [x] Test inline preview rendering
- [x] Test acceptance keys work correctly
- [x] Document auto mode in `docs/auto-mode.md`

**Success criteria**:
- ✅ Auto mode triggers after 500ms idle input
- ✅ Inline preview displays as gray text (Zsh: POSTDISPLAY, Bash: ANSI)
- ✅ Tab key accepts suggestion smoothly
- ✅ Cancellation works (no stale completions)
- ✅ 6 auto mode tests passing

#### Phase 4: Windows Auto Mode (1-2 weeks) ✅ COMPLETED

**Goal**: Implement auto mode for PowerShell

**Tasks**:
- [x] Create NudgePredictor class (PowerShell 7.2+)
- [x] Implement `GetSuggestion` method calling `nudge complete`
- [x] Register predictor with SubsystemManager
- [x] Configure PSReadLine options (PredictionSource, accept keys)
- [x] Implement fallback for PowerShell 5.1 (manual mode only)
- [x] Document PowerShell auto mode setup
- [x] Update `integration.ps1` with auto mode logic
- [x] Update installation script to install NudgePredictor module

**Success criteria**:
- ✅ Auto mode works in PowerShell 7.2+ with PSReadLine
- ✅ Tab accepts suggestion
- ✅ Fallback mode works in PowerShell 5.1 (manual mode)
- ✅ Module installs correctly via install script

#### Phase 5: Polish and Documentation (1 week) ✅ COMPLETED

**Goal**: Final touches, comprehensive docs, release prep

**Tasks**:
- [x] Update README.md with new features
- [x] Update README_zh.md (Chinese version)
- [x] Write migration guide for existing users
- [x] Create troubleshooting guide
- [ ] Record demo GIFs/videos for auto mode
- [x] Run full test suite on all platforms
- [ ] Performance benchmarking (manual vs auto mode)
- [x] Review and clean up TODO comments in code
- [x] Prepare release notes
- [x] Version bump to 0.3.0

**Success criteria**:
- Documentation covers all new features
- All platforms tested and working
- Migration path is clear for users
- Release notes are comprehensive

### Backward Compatibility

**Configuration files**:
- All existing `config.yaml` files remain valid
- New `trigger` section is optional (defaults to manual mode)
- Old shell integration scripts continue to work (but deprecated)

**CLI interface**:
- All existing commands unchanged (`daemon`, `complete`, `start`, `stop`, `status`)
- New commands are additive (`info`, `setup`)

**Deprecation warnings**:
- If user runs with old shell integration script, show:
  ```
  Warning: You are using a deprecated integration script.
  Please run 'nudge setup bash' to upgrade to the new version.
  ```

### Security Considerations

**FFI Safety**:
- All C ABI functions validate pointer arguments (null checks)
- CString conversions handle invalid UTF-8 gracefully
- Panics are caught at FFI boundary (`std::panic::catch_unwind`)

**Shell injection**:
- User input (buffer, cwd) is never executed as shell commands
- LLM responses are sanitized before returning to shell

**File permissions**:
- Config files created with mode 0600 (user-only read/write)
- Socket files created with mode 0700 (user-only access)
- PID files created with mode 0644 (world-readable)

**API keys**:
- Continue using existing sanitizer for context sent to LLM
- Never log API keys (redact in debug output)

### Open Questions

1. **Dynamic library distribution**:
   - Should `libnudge.so` be bundled in releases, or built locally on install?
   - If bundled, need separate builds for different glibc versions?

2. **Auto mode UX details**:
   - Should auto mode trigger on empty buffer with just CWD context?
   - How to handle multi-line commands (Bash `\` line continuation)?

3. **PowerShell 5.1 fallback**:
   - Is polling-based auto mode worth implementing, or just disable auto for old PowerShell?

4. **Shell detection reliability**:
   - Current method checks `$SHELL` env var, but what if user launches different shell?
   - Should we detect shell from parent process name?

### Summary

This design provides a comprehensive solution to nudge's cross-platform and auto mode challenges:

**Key improvements**:
- ✅ Centralized platform logic in Rust (eliminates path duplication)
- ✅ Platform-optimal implementations (Unix dynamic library, Windows enhanced CLI)
- ✅ True auto mode with inline preview (Unix)
- ✅ Simplified installation with `nudge setup`
- ✅ Robust fallback strategy (library → CLI → direct LLM)
- ✅ Backward compatible with existing deployments
- ✅ Comprehensive testing and documentation

**Trade-offs**:
- Increased code complexity in Unix implementation (FFI layer)
- Platform-specific auto mode implementations (not 100% feature parity)
- Longer development time (5-7 weeks total)

**Recommended next steps**:
1. Review and approve this design
2. Create feature branch for Phase 1 work
3. Begin implementing centralized platform logic
4. Iterate with user feedback after each phase

---

**End of Design Document**
