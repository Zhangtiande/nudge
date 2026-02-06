# CLI Reference

## Commands

| Command | Description |
|---------|-------------|
| `nudge start` | Start daemon in background |
| `nudge stop` | Stop running daemon |
| `nudge restart` | Restart daemon |
| `nudge status` | Check daemon status |
| `nudge info` | Show runtime information |
| `nudge doctor` | Diagnose shell integration health |
| `nudge setup` | Configure shell integration |
| `nudge daemon` | Run daemon (internal) |
| `nudge complete` | Request completion (internal) |
| `nudge diagnose` | Request error diagnosis (internal) |

## nudge start

Start the daemon in background.

```bash
nudge start
```

## nudge stop

Stop the running daemon.

```bash
nudge stop
```

## nudge restart

Restart the daemon. Use after configuration changes.

```bash
nudge restart
```

## nudge status

Check if daemon is running.

```bash
nudge status
```

**Exit codes:**
- `0` - Running
- `1` - Not running

## nudge info

Display runtime information.

```bash
# Human-readable output
nudge info

# JSON output
nudge info --json

# Get specific field
nudge info --field config_dir
nudge info --field shell_type
nudge info --field daemon_status
nudge info --field zsh_ghost_owner
nudge info --field zsh_overlay_backend
```

**Available fields:**
- `platform` - Platform name (e.g., "linux-x86_64")
- `config_dir` - Configuration directory
- `config_file` - User config file path
- `default_config_file` - Built-in config template path
- `socket_path` - IPC socket/pipe path
- `integration_script` - Generated shell integration script path
- `daemon_status` - Daemon status
- `shell_type` - Detected shell
- `trigger_mode` - Trigger mode (`manual` or `auto`)
- `trigger_hotkey` - Manual trigger key
- `auto_delay_ms` - Auto mode delay in milliseconds
- `zsh_ghost_owner` - Effective Zsh ghost owner strategy
- `zsh_overlay_backend` - Overlay backend (`message` or `rprompt`)
- `diagnosis_enabled` - Diagnosis feature status

## nudge setup

Configure shell integration.

```bash
# Auto-detect shell
nudge setup

# Specify shell
nudge setup zsh
nudge setup bash
nudge setup powershell

# Force reinstall
nudge setup --force
```

## nudge doctor

Diagnose shell integration health and key bindings.

```bash
# Auto target (defaults to zsh)
nudge doctor

# Explicit zsh target
nudge doctor zsh
```

Checks include:
- Config summary (`trigger.mode`, `zsh_ghost_owner`, `zsh_overlay_backend`)
- Zsh availability and integration script syntax
- Key binding snapshot (`Tab`, `Ctrl+G`, `Right`, `Alt+Right`, `Ctrl+Right`, `F1`)
- Hook presence (`line-pre-redraw`, `line-finish`)
- Daemon completion latency sample (`p50`, `p95`)

## nudge daemon

Run the daemon process. Usually called internally.

```bash
# Run in foreground (for debugging)
nudge daemon --foreground

# Fork and return immediately
nudge daemon --fork
```

## nudge complete

Request a completion. Called by shell integration scripts.

```bash
nudge complete \
  --buffer "git sta" \
  --cursor 7 \
  --cwd /path/to/project \
  --session bash-12345 \
  --last-exit-code 0 \
  --format plain
```

**Options:**
- `--buffer` - Current input buffer (required)
- `--cursor` - Cursor position (required)
- `--cwd` - Working directory (required)
- `--session` - Session ID (required)
- `--last-exit-code` - Last command exit code
- `--format` - Output format: `plain` or `json`

## nudge diagnose

Request error diagnosis. Called by shell integration scripts.

```bash
nudge diagnose \
  --exit-code 127 \
  --command "gti status" \
  --cwd /path/to/project \
  --session zsh-12345 \
  --stderr-file /tmp/stderr.txt \
  --format plain
```

**Options:**
- `--exit-code` - Failed command exit code (required)
- `--command` - Failed command text (required)
- `--cwd` - Working directory (required)
- `--session` - Session ID (required)
- `--stderr-file` - Path to captured stderr
- `--error-record` - PowerShell error record (JSON)
- `--format` - Output format: `plain` or `json`

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SMARTSHELL_CONFIG` | Override config file path |
| `RUST_LOG` | Override log level |

```bash
# Debug logging
RUST_LOG=nudge=debug nudge daemon --foreground

# Trace specific module
RUST_LOG=nudge::daemon::llm=trace nudge daemon --foreground
```

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Configuration error |
| `3` | IPC error |

## Common Workflows

### Initial Setup

```bash
nudge setup
source ~/.bashrc  # or ~/.zshrc
```

### Daily Usage

```bash
nudge start
# Use Ctrl+E for completions
nudge stop
```

### Troubleshooting

```bash
nudge info
nudge status
nudge daemon --foreground
RUST_LOG=nudge=debug nudge daemon --foreground
```

### Configuration Changes

```bash
# Edit config
$EDITOR $(nudge info --field config_file)

# Apply changes
nudge restart
```

## See Also

- [Configuration Reference](configuration.md)
- [Auto Mode Guide](auto-mode.md)
- [Installation Guide](installation.md)
