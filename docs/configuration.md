# Configuration Reference

This document describes all configuration options available in Nudge.

## Configuration File Location

| Platform | Path |
|----------|------|
| Linux | `~/.config/nudge/config.yaml` |
| macOS | `~/Library/Application Support/nudge/config.yaml` |
| Windows | `%APPDATA%\nudge\config\config.yaml` |

Use `nudge info` to view your configuration paths:

```bash
nudge info --field config_file
```

## Full Configuration Schema

```yaml
# LLM Configuration
model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"
  api_key: null
  api_key_env: null
  timeout_ms: 5000

# Context Settings
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

# Trigger Settings
trigger:
  mode: "manual"
  hotkey: "\\C-e"
  auto_delay_ms: 500
  zsh_ghost_owner: "auto"

# Cache Settings
cache:
  capacity: 1024
  prefix_bytes: 80
  ttl_auto_ms: 3000
  ttl_manual_ms: 15000
  ttl_negative_ms: 2000
  stale_ratio: 0.8

# Error Diagnosis
diagnosis:
  enabled: true
  timeout_ms: 5000

# Plugins
plugins:
  git:
    enabled: true
    depth: "standard"
    recent_commits: 5
    priority: 50
  docker:
    enabled: true
    timeout_ms: 100
    priority: 45
  node:
    enabled: true
    timeout_ms: 100
    priority: 45
  python:
    enabled: true
    timeout_ms: 100
    priority: 45
  rust:
    enabled: true
    timeout_ms: 100
    priority: 45

# Privacy
privacy:
  sanitize_enabled: true
  custom_patterns: []
  block_dangerous: true
  custom_blocked: []

# Logging
log:
  level: "info"
  file_enabled: false
```

## Configuration Sections

### Model

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `endpoint` | string | `http://localhost:11434/v1` | LLM API endpoint |
| `model_name` | string | `codellama:7b` | Model identifier |
| `api_key` | string | null | Direct API key |
| `api_key_env` | string | null | Environment variable for API key |
| `timeout_ms` | integer | 5000 | Request timeout (ms) |

**Examples:**

```yaml
# Ollama (local)
model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"

# OpenAI
model:
  endpoint: "https://api.openai.com/v1"
  model_name: "gpt-4o-mini"
  api_key_env: "OPENAI_API_KEY"

# Alibaba DashScope
model:
  endpoint: "https://dashscope.aliyuncs.com/compatible-mode/v1"
  model_name: "qwen-coder-plus"
  api_key_env: "DASHSCOPE_API_KEY"
```

### Context

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `history_window` | integer | 20 | Recent commands to include |
| `include_cwd_listing` | boolean | true | Include directory files |
| `include_exit_code` | boolean | true | Include last exit code |
| `include_system_info` | boolean | true | Include OS, arch, shell info |
| `similar_commands_enabled` | boolean | true | Enable similar command search |
| `similar_commands_window` | integer | 200 | History entries to search |
| `similar_commands_max` | integer | 5 | Max similar commands |
| `max_files_in_listing` | integer | 50 | Max files from CWD |
| `max_total_tokens` | integer | 4000 | Max context tokens |

**Priority Configuration:**

Higher priority = kept longer during truncation (1-100).

| Source | Default | Description |
|--------|---------|-------------|
| `history` | 80 | Command history |
| `cwd_listing` | 60 | Directory files |
| `plugins` | 40 | Plugin context |

### Trigger

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `mode` | string | `manual` | `manual` or `auto` |
| `hotkey` | string | `\C-e` | Manual trigger key |
| `auto_delay_ms` | integer | 500 | Auto mode debounce |
| `zsh_ghost_owner` | string | `auto` | Zsh ghost text owner: `auto`, `nudge`, or `autosuggestions` |

When `zsh_ghost_owner` resolves to `autosuggestions`, Nudge keeps `Tab` for autosuggestions and uses `Ctrl+G` to accept Nudge overlay/diagnosis suggestions.

### Cache

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `capacity` | integer | 1024 | Max cache entries (LRU) |
| `prefix_bytes` | integer | 80 | Max bytes of prefix for key hashing |
| `ttl_auto_ms` | integer | 3000 | Auto mode TTL (ms) |
| `ttl_manual_ms` | integer | 15000 | Manual mode TTL (ms) |
| `ttl_negative_ms` | integer | 2000 | Negative cache TTL (ms) |
| `stale_ratio` | float | 0.8 | Stale-while-revalidate threshold |

### Diagnosis

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | true | Enable error diagnosis |
| `timeout_ms` | integer | 5000 | Diagnosis timeout (ms) |

When enabled, Nudge analyzes failed commands with full project context:
- System information (OS, architecture, shell)
- Command history (session-based)
- Project plugins (Git, Node, Python, Rust, Docker)
- Directory listing

### Plugins

All plugins share common options:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | true | Enable plugin |
| `timeout_ms` | integer | 100 | Collection timeout |
| `priority` | integer | 45-50 | Truncation priority |

**Git Plugin:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `depth` | string | `standard` | `light`, `standard`, or `detailed` |
| `recent_commits` | integer | 5 | Commits to include |

Depth levels:
- `light`: Branch name, clean/dirty status (~20ms)
- `standard`: + staged files, recent commits (~35ms)
- `detailed`: + unstaged files, diff stats (~50ms)

### Privacy

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `sanitize_enabled` | boolean | true | Redact sensitive data |
| `custom_patterns` | array | [] | Custom regex patterns |
| `block_dangerous` | boolean | true | Warn on dangerous commands |
| `custom_blocked` | array | [] | Custom blocked patterns |

**Built-in sanitization:**
- API keys (OpenAI, GitHub, AWS)
- Bearer tokens
- Passwords in CLI flags
- URL credentials
- Private keys

**Built-in dangerous patterns:**
- `rm -rf /`, `rm -rf ~`
- `mkfs`, `dd if=`
- Fork bombs

### Logging

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `level` | string | `info` | `trace`, `debug`, `info`, `warn`, `error` |
| `file_enabled` | boolean | false | Enable file logging |

Log file locations:
- Linux: `~/.local/share/nudge/logs/`
- macOS: `~/Library/Application Support/nudge/logs/`
- Windows: `%LOCALAPPDATA%\nudge\data\logs\`

## Example Configurations

### Minimal (Performance)

```yaml
context:
  history_window: 5
  include_cwd_listing: false
  max_total_tokens: 1000
plugins:
  git:
    depth: "light"
```

### Maximum Privacy

```yaml
privacy:
  sanitize_enabled: true
  block_dangerous: true
  custom_patterns:
    - "my-company-secret-\\d+"
  custom_blocked:
    - "DROP TABLE"
```

### Auto Mode (Zsh)

```yaml
trigger:
  mode: auto
  auto_delay_ms: 400
  zsh_ghost_owner: auto
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SMARTSHELL_CONFIG` | Override config file path |
| `RUST_LOG` | Override log level (e.g., `nudge=debug`) |

## See Also

- [CLI Reference](cli-reference.md)
- [Auto Mode Guide](auto-mode.md)
- [Installation Guide](installation.md)
