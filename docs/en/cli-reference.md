# CLI Reference

[English](cli-reference.md) | [中文](../zh/cli-reference.md)

Practical command reference for day-to-day Nudge operation.

## Quick Examples

```bash
nudge status          # Is the daemon running?
nudge info            # Show config summary
nudge doctor zsh      # Check Zsh integration health
nudge restart         # Restart daemon with latest config
```

## Commands

### `nudge daemon [--foreground|--fork]`

Start the background daemon that handles completion requests via IPC.

- `--foreground`: Run in the current terminal (useful for debugging with `RUST_LOG=debug`)
- `--fork`: Fork to background (default when started via `nudge start`)

### `nudge start`

Start the daemon in background mode. Equivalent to `nudge daemon --fork`.

### `nudge stop`

Stop the running daemon by sending a shutdown signal.

### `nudge restart`

Stop and then start the daemon. Use after config changes.

### `nudge status`

Print whether the daemon is running and its PID.

### `nudge complete`

Request a completion from the daemon. This is called by shell integration scripts; you rarely need to invoke it directly.

```bash
nudge complete \
  --buffer "git st" \
  --cursor 6 \
  --cwd /path/to/project \
  --session "zsh-12345" \
  --shell-mode "zsh-inline" \
  --format plain
```

**Parameters**:

| Flag | Required | Description |
|---|---|---|
| `--buffer` | Yes | Current command line content |
| `--cursor` | Yes | Cursor position (byte offset) |
| `--cwd` | Yes | Working directory |
| `--session` | Yes | Session identifier for continuity |
| `--shell-mode` | No | Hint from integration: `zsh-inline`, `zsh-auto`, `bash-inline`, `bash-popup`, `ps-inline`, `cmd-inline` |
| `--format` | No | Output format (see below) |
| `--last-exit-code` | No | Exit code of previous command |

**Output formats** (`--format`):

| Format | Output | Use case |
|---|---|---|
| `plain` | Single suggestion string | Inline apply (Ctrl+E path) |
| `list` | Tab-separated rows: `risk\tcommand\twarning\twhy\tdiff` | Popup selector (Alt+/ path) |
| `json` | JSON object with `suggestion`, `warning`, `candidates` | Programmatic consumption |

### `nudge info [--json] [--field <name>]`

Show runtime information about the current Nudge installation.

- `--json`: Output as JSON object
- `--field <name>`: Print a single field value

**Common `--field` keys**:

| Key | Returns |
|---|---|
| `config_dir` | Config directory path |
| `config_file` | User config file path |
| `default_config_file` | Default config file path |
| `socket_path` | IPC socket/pipe path |
| `integration_script` | Shell integration script path |
| `daemon_status` | `running` or `stopped` |
| `shell_type` | Detected shell type |
| `trigger_mode` | `manual` or `auto` |
| `trigger_hotkey` | Current hotkey binding |
| `zsh_ghost_owner` | `auto`, `nudge`, or `autosuggestions` |
| `zsh_overlay_backend` | `message` or `rprompt` |
| `diagnosis_enabled` | `true` or `false` |
| `interactive_commands` | Comma-separated list |

### `nudge doctor [zsh|bash]`

Run integration health checks for a specific shell.

```bash
nudge doctor zsh
```

**What it checks**:

- Shell integration script is sourced
- Key bindings are registered (`Ctrl+E`, `Tab`, `Alt+/`, etc.)
- Hooks are installed (`precmd`, `preexec`, etc.)
- Daemon is reachable
- Config values are consistent

**Reading the output**: Each check prints `OK` or `WARN` with a brief explanation. If you see warnings, run `nudge setup <shell> --force` to refresh integration.

### `nudge setup [bash|zsh|powershell] [--force]`

Write the shell integration script and add the `source` hook to your profile.

- `bash`: Writes to `.bashrc`
- `zsh`: Writes to `.zshrc`
- `powershell`: Writes to PowerShell `$PROFILE`
- `--force`: Overwrite existing integration files

### `nudge diagnose`

Analyze a failed command and suggest a fix. Called automatically by shell integration when `diagnosis.enabled: true`.

```bash
nudge diagnose \
  --exit-code 1 \
  --command "cargo build" \
  --cwd /path/to/project \
  --session "zsh-12345" \
  --stderr-file /tmp/nudge_stderr_12345 \
  --format plain
```

**Parameters**:

| Flag | Required | Description |
|---|---|---|
| `--exit-code` | Yes | Exit code of the failed command |
| `--command` | Yes | The command that failed |
| `--cwd` | Yes | Working directory |
| `--session` | Yes | Session identifier |
| `--stderr-file` | No | Path to captured stderr file |
| `--error-record` | No | JSON error record (PowerShell) |
| `--format` | No | Output format: `plain` or `json` |

**Output** (plain format): Two lines — first is the diagnosis message, second is the suggested fix command. Shell integration shows the diagnosis and lets you press Tab to accept the fix.

## Typical Workflows

**Initial setup check**:

```bash
nudge setup zsh --force
nudge restart
nudge status
nudge info
nudge doctor zsh
```

**Debug daemon behavior**:

```bash
RUST_LOG=debug nudge daemon --foreground
# In another terminal, trigger completion to see logs
```

**Check cache behavior**:

```bash
RUST_LOG=debug nudge daemon --foreground
# Look for "cache hit" / "cache miss" / "stale-revalidate" in output
```

## Boundaries

- `nudge complete` is an integration/internal path; prefer shell key bindings in normal use
- `--shell-mode` is a hint from integration scripts; do not rely on undocumented values
- `nudge diagnose` output format may evolve across versions
