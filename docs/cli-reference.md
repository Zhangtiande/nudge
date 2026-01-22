# Nudge CLI Reference

This document provides a comprehensive reference for all Nudge CLI commands.

## Table of Contents

- [Global Options](#global-options)
- [Commands](#commands)
  - [nudge daemon](#nudge-daemon)
  - [nudge complete](#nudge-complete)
  - [nudge start](#nudge-start)
  - [nudge stop](#nudge-stop)
  - [nudge restart](#nudge-restart)
  - [nudge status](#nudge-status)
  - [nudge info](#nudge-info)
  - [nudge setup](#nudge-setup)
  - [nudge config](#nudge-config)

---

## Global Options

```bash
nudge --help     # Show help information
nudge --version  # Show version information
```

---

## Commands

### nudge daemon

Start the Nudge daemon process.

**Usage:**
```bash
nudge daemon [OPTIONS]
```

**Options:**
- `--foreground` - Run in foreground (don't daemonize). Useful for debugging.
- `--fork` - Fork and return immediately. Used for shell lazy-loading.

**Examples:**

```bash
# Start daemon in foreground (blocks terminal, shows logs)
nudge daemon --foreground

# Start daemon with fork (returns immediately)
nudge daemon --fork

# Start daemon in background (default behavior)
nudge daemon
```

**Notes:**
- The daemon listens for completion requests via IPC (Unix Domain Socket on Unix, Named Pipe on Windows)
- Only one daemon instance can run at a time
- Use `nudge start` for a simpler way to start the daemon in background

---

### nudge complete

Request a command completion from the daemon. This command is typically called by shell integration scripts, not directly by users.

**Usage:**
```bash
nudge complete --buffer <BUFFER> --cursor <CURSOR> --cwd <CWD> --session <SESSION> [OPTIONS]
```

**Required Options:**
- `--buffer <BUFFER>` - Current input buffer content
- `--cursor <CURSOR>` - Cursor position within buffer (0-indexed)
- `--cwd <CWD>` - Current working directory
- `--session <SESSION>` - Session identifier (e.g., "bash-12345")

**Optional Options:**
- `--last-exit-code <CODE>` - Exit code of the last executed command
- `--format <FORMAT>` - Output format: `plain` (default) or `json`

**Examples:**

```bash
# Request completion (typically called by shell integration)
nudge complete \
  --buffer "git com" \
  --cursor 7 \
  --cwd /home/user/project \
  --session bash-12345 \
  --last-exit-code 0

# Request completion with JSON output
nudge complete \
  --buffer "docker ps" \
  --cursor 9 \
  --cwd /home/user \
  --session zsh-67890 \
  --format json
```

**Output (plain format):**
```
git commit -m "feat: add new feature"
```

**Output (json format):**
```json
{
  "suggestion": "git commit -m \"feat: add new feature\"",
  "is_dangerous": false,
  "warning": null
}
```

---

### nudge start

Start the Nudge daemon in the background.

**Usage:**
```bash
nudge start
```

**Examples:**

```bash
# Start daemon
nudge start
```

**Output:**
```
Starting Nudge daemon...
✓ Daemon started successfully
```

**Notes:**
- This is equivalent to `nudge daemon --fork` but with better user feedback
- If daemon is already running, shows an error message
- The daemon will continue running until stopped with `nudge stop`

---

### nudge stop

Stop the running Nudge daemon.

**Usage:**
```bash
nudge stop
```

**Examples:**

```bash
# Stop daemon
nudge stop
```

**Output:**
```
Stopping Nudge daemon...
✓ Daemon stopped successfully
```

**Notes:**
- Sends SIGTERM to the daemon process (Unix) or terminates the process (Windows)
- If no daemon is running, shows an error message

---

### nudge restart

Restart the Nudge daemon (stop + start).

**Usage:**
```bash
nudge restart
```

**Examples:**

```bash
# Restart daemon (useful after config changes)
nudge restart
```

**Output:**
```
Stopping Nudge daemon...
✓ Daemon stopped successfully
Starting Nudge daemon...
✓ Daemon started successfully
```

**Notes:**
- Useful after modifying configuration files
- Equivalent to running `nudge stop && nudge start`

---

### nudge status

Check the status of the Nudge daemon.

**Usage:**
```bash
nudge status
```

**Examples:**

```bash
# Check daemon status
nudge status
```

**Output (running):**
```
Nudge daemon is running
  PID: 12345
  Socket: /home/user/.config/nudge/nudge.sock
```

**Output (not running):**
```
Nudge daemon is not running
```

**Exit Codes:**
- `0` - Daemon is running
- `1` - Daemon is not running

---

### nudge info

Display runtime information including paths, configuration, and daemon status.

**Usage:**
```bash
nudge info [OPTIONS]
```

**Options:**
- `--json` - Output as JSON
- `--field <FIELD>` - Get specific field value

**Available Fields:**
- `platform` - Platform name (e.g., "linux-x86_64", "macos-aarch64", "windows-x86_64")
- `config_dir` - Configuration directory path
- `config_file` - User configuration file path
- `default_config_file` - Default configuration file path
- `socket_path` - IPC socket/pipe path
- `integration_script` - Shell integration script path
- `daemon_status` - Daemon status string
- `shell_type` - Detected shell type (bash, zsh, powershell, cmd)

**Examples:**

```bash
# Show all runtime information (human-readable)
nudge info
```

**Output:**
```
Nudge Runtime Information
=========================

Platform:             linux-x86_64
Config Directory:     /home/user/.config/nudge
Config File:          /home/user/.config/nudge/config.yaml
Default Config:       /home/user/.config/nudge/config/config.default.yaml
Socket Path:          /home/user/.config/nudge/nudge.sock
Integration Script:   /home/user/.config/nudge/integration.bash
Daemon Status:        Running (socket exists)
Shell Type:           bash
```

```bash
# Get JSON output (for scripting)
nudge info --json
```

**Output:**
```json
{
  "platform": "linux-x86_64",
  "config_dir": "/home/user/.config/nudge",
  "config_file": "/home/user/.config/nudge/config.yaml",
  "default_config_file": "/home/user/.config/nudge/config/config.default.yaml",
  "socket_path": "/home/user/.config/nudge/nudge.sock",
  "integration_script": "/home/user/.config/nudge/integration.bash",
  "daemon_status": "Running (socket exists)",
  "shell_type": "bash"
}
```

```bash
# Get specific field (useful in scripts)
nudge info --field config_dir
# Output: /home/user/.config/nudge

nudge info --field shell_type
# Output: bash

nudge info --field daemon_status
# Output: Running (socket exists)
```

**Use Cases:**
- Check where configuration files are located
- Verify shell integration script path
- Get daemon status for monitoring scripts
- Debug installation issues

---

### nudge setup

Automatically setup shell integration for Nudge.

**Usage:**
```bash
nudge setup [SHELL] [OPTIONS]
```

**Arguments:**
- `[SHELL]` - Shell type to setup (bash, zsh, powershell). Auto-detected if not specified.

**Options:**
- `--force` - Force reinstall even if already configured

**Examples:**

```bash
# Auto-detect shell and setup integration
nudge setup
```

**Output:**
```
Setting up Nudge for bash...

✓ Installed integration script to /home/user/.config/nudge/integration.bash
✓ Added Nudge integration to /home/user/.bashrc
✓ Daemon is already running

✓ Setup complete!

To start using Nudge:
  1. Restart your terminal or run: source ~/.bashrc
  2. Press Ctrl+E to get AI-powered command suggestions
```

```bash
# Setup for specific shell
nudge setup zsh

# Force reinstall (overwrites existing integration)
nudge setup --force

# Setup PowerShell on Windows
nudge setup powershell
```

**What it does:**
1. Detects your shell (or uses specified shell)
2. Installs the integration script to your config directory
3. Adds a source line to your shell profile (~/.bashrc, ~/.zshrc, or $PROFILE)
4. Starts the daemon if not already running

**Shell Profile Locations:**
- **Bash**: `~/.bashrc`
- **Zsh**: `~/.zshrc`
- **PowerShell**: `$PROFILE` (typically `~/Documents/PowerShell/Microsoft.PowerShell_profile.ps1`)

**Notes:**
- If integration is already configured, use `--force` to reinstall
- CMD shell is not supported (use PowerShell instead)
- After setup, restart your shell or source your profile to activate

---

### nudge config

**DEPRECATED:** Use `nudge info` instead.

Display configuration paths and optionally show full configuration.

**Usage:**
```bash
nudge config [OPTIONS]
```

**Options:**
- `--show` - Show full configuration (not just paths)

**Examples:**

```bash
# Show configuration paths
nudge config

# Show full configuration
nudge config --show
```

**Migration:**
```bash
# Old command
nudge config

# New equivalent
nudge info

# Old command with --show
nudge config --show

# New equivalent (use info + read config file)
nudge info --field config_file
cat $(nudge info --field config_file)
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SMARTSHELL_CONFIG` | Override config file path |
| `RUST_LOG` | Override log level (e.g., `nudge=debug`, `nudge::daemon=trace`) |

**Examples:**

```bash
# Use custom config file
export SMARTSHELL_CONFIG=/path/to/custom/config.yaml
nudge start

# Enable debug logging
export RUST_LOG=nudge=debug
nudge daemon --foreground

# Enable trace logging for specific module
export RUST_LOG=nudge::daemon::llm=trace
nudge daemon --foreground
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Configuration error |
| `3` | IPC error (daemon not running or connection failed) |

---

## Common Workflows

### Initial Setup

```bash
# 1. Install nudge binary (via install script or manual)
# 2. Setup shell integration
nudge setup

# 3. Restart shell or source profile
source ~/.bashrc  # or ~/.zshrc, or restart PowerShell

# 4. Test completion
# Type a partial command and press Ctrl+E
```

### Daily Usage

```bash
# Start daemon (if not auto-started)
nudge start

# Use Ctrl+E in your shell to get completions

# Check daemon status
nudge status

# Stop daemon when done
nudge stop
```

### Troubleshooting

```bash
# Check runtime information
nudge info

# Check daemon status
nudge status

# Restart daemon (after config changes)
nudge restart

# Run daemon in foreground to see logs
nudge daemon --foreground

# Enable debug logging
RUST_LOG=nudge=debug nudge daemon --foreground
```

### Configuration Management

```bash
# Find config file location
nudge info --field config_file

# Edit config file
$EDITOR $(nudge info --field config_file)

# Restart daemon to apply changes
nudge restart

# Verify configuration is loaded
nudge info
```

---

## See Also

- [Configuration Reference](configuration.md) - Detailed configuration options
- [README](../README.md) - Project overview and installation
- [GitHub Repository](https://github.com/Zhangtiande/nudge) - Source code and issues
