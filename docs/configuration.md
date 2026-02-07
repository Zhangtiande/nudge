# Configuration Reference

This document explains how to configure Nudge safely without editing source code.

## Config Files

- User overrides: `~/.nudge/config/config.yaml`
- Shipped defaults: `~/.nudge/config/config.default.yaml`
- Env override file path: `NUDGE_CONFIG`

Load order: defaults -> `config.default.yaml` -> `config.yaml`.

## Minimal Config

Local model:

```yaml
model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"

trigger:
  mode: manual
```

Remote model:

```yaml
model:
  endpoint: "https://api.openai.com/v1"
  model_name: "gpt-4o-mini"
  api_key_env: "OPENAI_API_KEY"
```

## High-Impact Options

### `model`

- `endpoint`: OpenAI-compatible endpoint
- `model_name`: model id
- `api_key` / `api_key_env`: auth
- `timeout_ms`: request timeout

### `trigger`

- `mode`: `manual` or `auto`
- `hotkey`: default `\C-e`
- `zsh_ghost_owner`: `auto|nudge|autosuggestions`
- `zsh_overlay_backend`: `message|rprompt`

### `privacy`

- `sanitize_enabled`: remove secrets from context
- `block_dangerous`: risk warning gate
- `custom_patterns`, `custom_blocked`: org-specific rules

### `diagnosis`

- `enabled`: enable failed-command diagnosis
- `capture_stderr`: capture stderr for analysis (zsh path)
- `interactive_commands`: skip capture for interactive tools

### `log`

- `level`: `trace|debug|info|warn|error`
- `file_enabled`: write logs to `~/.nudge/logs`

## Practical Profiles

Latency-first:

```yaml
context:
  history_window: 10
  include_cwd_listing: false
plugins:
  docker:
    enabled: false
```

Safety-first:

```yaml
privacy:
  sanitize_enabled: true
  block_dangerous: true
diagnosis:
  enabled: true
```

## Validate and Observe

```bash
nudge info --json
nudge doctor zsh
nudge doctor bash
```

## Boundaries

- Bash/CMD do not gain true auto ghost-text mode from config alone
- Over-aggressive custom regex may hide useful context
