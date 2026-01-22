# Nudge Configuration Reference

This document describes all configuration options available in nudge.

## Checking Configuration

Use `nudge info` to view runtime configuration and paths:

```bash
# Human-readable output
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
# JSON output (for scripting)
nudge info --json
```

```bash
# Get specific field
nudge info --field config_dir
# Output: /home/user/.config/nudge

nudge info --field shell_type
# Output: bash
```

For full CLI reference, see [CLI Reference](cli-reference.md).

## Configuration File Location

Nudge looks for configuration in the following order:

1. `$SMARTSHELL_CONFIG` environment variable (if set)
2. Platform-specific default location:
   - **macOS**: `~/Library/Application Support/nudge/config.yaml`
   - **Linux**: `~/.config/nudge/config.yaml`
   - **Windows**: `%APPDATA%\nudge\config\config.yaml`
3. Built-in defaults (if no config file exists)

### Related Files

| File | macOS | Linux | Windows |
|------|-------|-------|---------|
| Config | `~/Library/Application Support/nudge/config.yaml` | `~/.config/nudge/config.yaml` | `%APPDATA%\nudge\config\config.yaml` |
| IPC | `~/Library/Application Support/nudge/nudge.sock` | `~/.config/nudge/nudge.sock` | `\\.\pipe\nudge_{username}` |
| PID | `~/Library/Application Support/nudge/nudge.pid` | `~/.config/nudge/nudge.pid` | `%APPDATA%\nudge\config\nudge.pid` |
| Logs | `~/Library/Application Support/nudge/logs/` | `~/.local/share/nudge/logs/` | `%LOCALAPPDATA%\nudge\data\logs\` |

**Note:** On Windows, IPC uses Named Pipes instead of Unix Domain Sockets.

## Configuration Schema

```yaml
# Model/LLM configuration
model:
  endpoint: "http://localhost:11434/v1"  # API endpoint URL
  model_name: "codellama:7b"              # Model identifier
  api_key: null                           # Direct API key (takes precedence)
  api_key_env: null                       # Environment variable for API key (fallback)
  timeout_ms: 5000                        # Request timeout in milliseconds

# Context collection settings
context:
  history_window: 20                      # Number of history commands to include
  include_cwd_listing: true               # Include current directory files
  include_exit_code: true                 # Include last command exit code
  include_system_info: true               # Include system information (OS, arch, shell, user)
  similar_commands_enabled: true          # Enable similar command search (like Bash Ctrl+R)
  similar_commands_window: 200            # Number of history commands to search
  similar_commands_max: 5                 # Maximum similar commands to return
  max_files_in_listing: 50                # Max files to list from CWD
  max_total_tokens: 4000                  # Max context tokens
  priorities:
    history: 80                           # History priority (1-100)
    cwd_listing: 60                       # CWD listing priority
    plugins: 40                           # Plugin data priority

# Plugin settings
plugins:
  git:
    enabled: true                         # Enable git context plugin
    depth: "standard"                     # Depth level: light/standard/detailed
    recent_commits: 5                     # Number of recent commits to include
    priority: 50                          # Plugin priority for truncation

# Trigger behavior
trigger:
  mode: "manual"                          # Trigger mode: manual/auto
  hotkey: "\\C-e"                         # Hotkey for manual trigger
  auto_delay_ms: 500                      # Delay for auto mode

# Privacy settings
privacy:
  sanitize_enabled: true                  # Enable sensitive data sanitization
  custom_patterns: []                     # Custom regex patterns for sanitization
  block_dangerous: true                   # Enable dangerous command warnings
  custom_blocked: []                      # Custom dangerous command patterns

# Logging settings
log:
  level: "info"                           # Log level: trace/debug/info/warn/error
  file_enabled: false                     # Enable file logging (daily rotation)

# System prompt (optional)
system_prompt: null                       # Override default system prompt
```

## Configuration Options

### Model Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `model.endpoint` | string | `http://localhost:11434/v1` | LLM API endpoint URL |
| `model.model_name` | string | `codellama:7b` | Model name/identifier |
| `model.api_key` | string? | `null` | Direct API key (takes precedence over `api_key_env`) |
| `model.api_key_env` | string? | `null` | Environment variable containing API key (fallback) |
| `model.timeout_ms` | integer | `5000` | Request timeout in milliseconds |

**Supported Endpoints:**

| Provider | Endpoint | API Key Required |
|----------|----------|------------------|
| Ollama (local) | `http://localhost:11434/v1` | No |
| OpenAI | `https://api.openai.com/v1` | Yes (`OPENAI_API_KEY`) |
| Azure OpenAI | Custom | Yes |
| llama.cpp | `http://localhost:8080/v1` | No |

### Context Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `context.history_window` | integer | `20` | Number of recent commands to include |
| `context.include_cwd_listing` | boolean | `true` | Whether to include CWD file listing |
| `context.include_exit_code` | boolean | `true` | Whether to include last exit code |
| `context.include_system_info` | boolean | `true` | Include system info (OS, architecture, shell, user) |
| `context.similar_commands_enabled` | boolean | `true` | Enable similar command search from history |
| `context.similar_commands_window` | integer | `200` | Number of history entries to search |
| `context.similar_commands_max` | integer | `5` | Maximum similar commands to return |
| `context.max_files_in_listing` | integer | `50` | Maximum files to include from CWD |
| `context.max_total_tokens` | integer | `4000` | Maximum total context tokens |

**System Information:**

When `include_system_info` is enabled, the LLM receives:
- Operating system type and version (e.g., "Windows 11 Build 22621", "Ubuntu 22.04")
- System architecture (e.g., "x86_64", "aarch64")
- Current shell type (e.g., "bash", "zsh", "powershell", "cmd")
- Current username

This helps the LLM provide platform-specific and shell-appropriate completions.

**Similar Commands Feature:**

When `similar_commands_enabled` is true and you type a command (≥3 characters), Nudge automatically searches your history for similar commands and includes them in the context. This works like Bash's `Ctrl+R` reverse search, but powered by LLM's semantic understanding.

Example: Typing `docker ps` will find similar commands like:
- `docker ps -a`
- `docker ps --format "table {{.ID}}\t{{.Names}}"`
- `docker inspect container_name`

The search uses keyword matching (ignoring common shell commands like cd, ls) and returns the most recent matches.

**Priority Configuration:**

Priority values range from 1-100. Higher values mean the data is kept longer during truncation.

| Source | Default Priority | Description |
|--------|------------------|-------------|
| `system_info` | 100 (implicit) | System information (never truncated) |
| `history` | 80 | Shell command history |
| `similar_commands` | 70 (implicit) | Similar commands from history |
| `cwd_listing` | 60 | Current directory files |
| `plugins` | 40 | Plugin-provided context |

**Note:** System information has the highest implicit priority and is never truncated. Similar commands are prioritized between history and CWD listing.

### Git Plugin Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `plugins.git.enabled` | boolean | `true` | Enable the git context plugin |
| `plugins.git.depth` | string | `standard` | Context depth level |
| `plugins.git.recent_commits` | integer | `5` | Number of recent commits |
| `plugins.git.priority` | integer? | `50` | Plugin priority override |

**Git Depth Levels:**

| Level | Commands | Data Collected | Timeout |
|-------|----------|----------------|---------|
| `light` | 2 | Branch name, clean/dirty status | ~20ms |
| `standard` | 4 | Light + staged files, recent commits | ~35ms |
| `detailed` | 5 | Standard + unstaged files, diff stats | ~50ms |

### Trigger Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `trigger.mode` | string | `manual` | Trigger mode (`manual` or `auto`) |
| `trigger.hotkey` | string | `\C-e` | Hotkey for manual trigger |
| `trigger.auto_delay_ms` | integer | `500` | Delay before auto-trigger |

**Trigger Modes:**

- `manual`: User presses hotkey to trigger completion
- `auto`: Completion triggers automatically after typing pause (future feature)

### Privacy Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `privacy.sanitize_enabled` | boolean | `true` | Enable sensitive data redaction |
| `privacy.custom_patterns` | array | `[]` | Custom regex patterns for sanitization |
| `privacy.block_dangerous` | boolean | `true` | Enable dangerous command warnings |
| `privacy.custom_blocked` | array | `[]` | Custom dangerous command patterns |

**Built-in Sanitization Patterns:**

- OpenAI API keys (`sk-...`)
- GitHub tokens (`ghp_...`, `gho_...`, `ghs_...`)
- AWS credentials (`AKIA...`)
- Bearer tokens
- CLI passwords (`--password=...`, `-p ...`)
- URL credentials (`user:pass@host`)
- Private keys (PEM format)
- Environment variable secrets

**Built-in Dangerous Patterns:**

- Recursive deletion (`rm -rf /`, `rm -rf ~`)
- Disk formatting (`mkfs`, `dd if=`)
- Fork bombs (`:(){:|:&};:`)

### Logging Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `log.level` | string | `info` | Log level: trace, debug, info, warn, error |
| `log.file_enabled` | boolean | `false` | Enable file logging with daily rotation |

**Log File Location:**

When `log.file_enabled` is `true`, logs are written to:
- **macOS**: `~/Library/Application Support/nudge/logs/nudge.log.YYYY-MM-DD`
- **Linux**: `~/.local/share/nudge/logs/nudge.log.YYYY-MM-DD`
- **Windows**: `%LOCALAPPDATA%\nudge\data\logs\nudge.log.YYYY-MM-DD`

Logs are rotated daily. Both console (stderr) and file output are enabled when file logging is on.

### System Prompt

You can override the default system prompt:

```yaml
system_prompt: |
  You are a terminal completion engine. Output only the command text.
  Do not include markdown formatting, comments, or explanations.
```

## Example Configurations

### Local Ollama Setup

```yaml
model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"
  timeout_ms: 5000
```

### OpenAI Setup (using environment variable)

```yaml
model:
  endpoint: "https://api.openai.com/v1"
  model_name: "gpt-3.5-turbo"
  api_key_env: "OPENAI_API_KEY"
  timeout_ms: 10000
```

### OpenAI Setup (using direct API key)

```yaml
model:
  endpoint: "https://api.openai.com/v1"
  model_name: "gpt-3.5-turbo"
  api_key: "sk-your-api-key-here"
  timeout_ms: 10000
```

**Note:** Using `api_key_env` is recommended for better security. Direct `api_key` is convenient for quick setup but exposes your key in the config file.

### Minimal Context (Performance Mode)

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
    - "internal-token-[a-f0-9]+"
  custom_blocked:
    - "DROP TABLE"
    - "TRUNCATE"
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SMARTSHELL_CONFIG` | Override config file path |
| `RUST_LOG` | Override log level from config (e.g., `nudge=debug`, `nudge::daemon=trace`) |

## Validation

Configuration is validated on load. The following constraints apply:

- `model.timeout_ms` must be > 0
- `context.history_window` must be > 0
- `context.max_total_tokens` must be > 0
- Custom regex patterns must be valid

Invalid configuration will result in a startup error with a descriptive message.
