# Configuration Reference

[English](configuration.md) | [中文](../zh/configuration.md)

This document explains how to configure Nudge without editing source code.

## Config Files

| File | Purpose |
|---|---|
| `~/.nudge/config/config.default.yaml` | Shipped defaults. Updated on upgrades. **Do not edit.** |
| `~/.nudge/config/config.yaml` | Your overrides. Preserved across upgrades. |

Env override: set `NUDGE_CONFIG` to point to a custom config path.

Load order: built-in Rust defaults → `config.default.yaml` → `config.yaml`. User overrides win via deep merge.

## Minimal Config

Local model (Ollama):

```yaml
model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"

trigger:
  mode: manual
```

Remote model (OpenAI):

```yaml
model:
  endpoint: "https://api.openai.com/v1"
  model_name: "gpt-4o-mini"
  api_key_env: "OPENAI_API_KEY"
```

## Complete Configuration Schema

### `model` — LLM Connection

| Key | Type | Default | Description |
|---|---|---|---|
| `endpoint` | string | `http://localhost:11434/v1` | OpenAI-compatible API endpoint |
| `model_name` | string | `codellama:7b` | Model identifier |
| `api_key` | string | _(none)_ | API key (direct, takes precedence over env) |
| `api_key_env` | string | _(none)_ | Environment variable name holding API key |
| `timeout_ms` | int | `5000` | Request timeout in milliseconds |

### `context` — What Gets Sent to the LLM

| Key | Type | Default | Description |
|---|---|---|---|
| `history_window` | int | `20` | Number of recent history entries |
| `include_cwd_listing` | bool | `true` | Include file listing of current directory |
| `include_exit_code` | bool | `true` | Include last command exit code |
| `include_system_info` | bool | `true` | Include OS, architecture, shell type |
| `similar_commands_enabled` | bool | `true` | Search history for similar commands |
| `similar_commands_window` | int | `200` | History depth for similarity search |
| `similar_commands_max` | int | `5` | Max similar commands returned |
| `max_files_in_listing` | int | `50` | Max files in directory listing |
| `max_total_tokens` | int | `4000` | Token budget for all context |
| `priorities.history` | int | `80` | Priority weight for history |
| `priorities.cwd_listing` | int | `60` | Priority weight for directory listing |
| `priorities.plugins` | int | `40` | Priority weight for plugin output |

**Context truncation**: When total context exceeds `max_total_tokens`, lower-priority items are removed first. Token estimation uses a word-based heuristic with 1.3x multiplier.

### `plugins` — Project Context

Each plugin follows the same pattern: `enabled`, `timeout_ms`, optional `priority`, and plugin-specific settings.

#### `plugins.git`

| Key | Type | Default | Description |
|---|---|---|---|
| `enabled` | bool | `true` | Enable Git context |
| `depth` | string | `standard` | `light` (branch only), `standard` (+staged/unstaged), `detailed` (+commits) |
| `recent_commits` | int | `5` | Commits shown in `detailed` mode |

Git operations have a strict 50ms internal timeout to prevent stalling.

#### `plugins.docker`

| Key | Type | Default | Description |
|---|---|---|---|
| `enabled` | bool | `true` | Enable Docker context |
| `timeout_ms` | int | `100` | Command execution timeout |
| `max_containers` | int | `10` | Max containers in context |
| `max_images` | int | `10` | Max images in context |
| `show_containers` | bool | `true` | Include running containers |
| `include_compose` | bool | `true` | Include docker-compose services |
| `include_dockerfile` | bool | `true` | Include Dockerfile preview (first 50 lines) |

#### `plugins.node`

| Key | Type | Default | Description |
|---|---|---|---|
| `enabled` | bool | `true` | Enable Node.js context |
| `timeout_ms` | int | `100` | File operation timeout |
| `max_dependencies` | int | `50` | Max dependencies listed |

Reads `package.json` for scripts and dependency information.

#### `plugins.rust`

| Key | Type | Default | Description |
|---|---|---|---|
| `enabled` | bool | `true` | Enable Rust/Cargo context |
| `timeout_ms` | int | `100` | File operation timeout |
| `max_dependencies` | int | `50` | Max dependencies listed |

Reads `Cargo.toml` for workspace and dependency information.

#### `plugins.python`

| Key | Type | Default | Description |
|---|---|---|---|
| `enabled` | bool | `true` | Enable Python context |
| `timeout_ms` | int | `100` | File operation timeout |
| `max_dependencies` | int | `50` | Max dependencies listed |

Reads `pyproject.toml`, `requirements.txt`, and detects virtual environments (uv, poetry, pip).

### `trigger` — How Completion Is Activated

| Key | Type | Default | Description |
|---|---|---|---|
| `mode` | string | `manual` | `manual` or `auto` |
| `hotkey` | string | `\C-e` | Readline-format key binding for manual trigger |
| `auto_delay_ms` | int | `500` | Debounce delay for auto mode |
| `zsh_ghost_owner` | string | `auto` | `auto`, `nudge`, or `autosuggestions` |
| `zsh_overlay_backend` | string | `message` | `message` or `rprompt` |

See [Auto Mode Guide](auto-mode.md) for detailed explanation of ghost ownership and overlay backends.

### `cache` — Suggestion Cache

| Key | Type | Default | Description |
|---|---|---|---|
| `capacity` | int | `1024` | Max cache entries (LRU eviction) |
| `prefix_bytes` | int | `80` | Max bytes of command prefix used for key hashing |
| `ttl_auto_ms` | int | `300000` | TTL for auto mode entries (5 min) |
| `ttl_manual_ms` | int | `600000` | TTL for manual mode entries (10 min) |
| `ttl_negative_ms` | int | `30000` | TTL for failed/empty results (30 sec) |
| `stale_ratio` | float | `0.8` | Stale-while-revalidate threshold (0.0–1.0) |

**Cache key**: `sk:v1:{prefix_hash}:{cwd_hash}:{git_hash}:{shell_mode}`. Any context change (directory, git state) automatically invalidates relevant entries.

**Stale-while-revalidate**: When an entry reaches `stale_ratio × TTL` age, it is returned immediately while a background refresh is triggered. This provides low-latency responses without serving stale data for too long.

### `privacy` — Sanitization and Safety

| Key | Type | Default | Description |
|---|---|---|---|
| `sanitize_enabled` | bool | `true` | Remove secrets from context before LLM call |
| `custom_patterns` | list | `[]` | Additional regex patterns to sanitize |
| `block_dangerous` | bool | `true` | Block dangerous commands (rm -rf, fork bombs) |
| `custom_blocked` | list | `[]` | Additional dangerous command patterns |

### `log` — Logging

| Key | Type | Default | Description |
|---|---|---|---|
| `level` | string | `info` | `trace`, `debug`, `info`, `warn`, `error` |
| `file_enabled` | bool | `false` | Write logs to `~/.nudge/logs/` with daily rotation |

### `diagnosis` — Error Diagnosis

| Key | Type | Default | Description |
|---|---|---|---|
| `enabled` | bool | `false` | Enable failed-command diagnosis |
| `capture_stderr` | bool | `true` | Capture stderr for analysis (Zsh) |
| `auto_suggest` | bool | `true` | Show inline fix suggestion (Tab to accept) |
| `max_stderr_size` | int | `4096` | Max stderr bytes sent to LLM |
| `timeout_ms` | int | `5000` | Diagnosis request timeout |
| `interactive_commands` | list | _(see below)_ | Commands that skip stderr capture |

Default `interactive_commands`: vim, nvim, vi, nano, emacs, code, ssh, telnet, mosh, top, htop, btop, less, more, man, fzf, sk, tmux, screen, python, python3, ipython, node, irb, psql, mysql, sqlite3, watch, tail.

### `system_prompt` — Custom LLM Prompt

Override the default system prompt sent to the LLM:

```yaml
system_prompt: |
  You are a helpful command-line assistant.
  Suggest commands that are safe and follow best practices.
  Always explain what the command does if it's complex.
```

## Practical Profiles

**Latency-first** — minimize context gathering:

```yaml
context:
  history_window: 10
  include_cwd_listing: false
plugins:
  docker:
    enabled: false
cache:
  ttl_manual_ms: 900000  # 15 min cache
```

**Safety-first** — maximize protection:

```yaml
privacy:
  sanitize_enabled: true
  block_dangerous: true
diagnosis:
  enabled: true
  capture_stderr: true
```

**Bandwidth-conscious** — local model with aggressive caching:

```yaml
model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"
cache:
  capacity: 2048
  ttl_auto_ms: 600000  # 10 min
context:
  max_total_tokens: 2000
```

## Validate and Observe

```bash
nudge info --json          # Full config dump
nudge info --field trigger_mode
nudge doctor zsh           # Integration health check
RUST_LOG=debug nudge daemon --foreground  # Watch cache hits/misses
```

## Boundaries

- Bash/CMD do not gain true auto ghost-text mode from config alone
- Over-aggressive custom regex may hide useful context
- `system_prompt` replaces the entire default prompt — include all instructions you need
- Cache is in-memory only; it resets when the daemon restarts
