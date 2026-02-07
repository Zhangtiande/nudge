# Nudge

> Nudge is a shell completion assistant for developers who want faster, safer command entry with project context.

[English](./README.md) | [中文](./README_zh.md)

[![CI](https://github.com/Zhangtiande/nudge/actions/workflows/ci.yml/badge.svg)](https://github.com/Zhangtiande/nudge/actions/workflows/ci.yml)
[![Release](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml/badge.svg)](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/Zhangtiande/nudge)](https://github.com/Zhangtiande/nudge/releases/latest)
[![License](https://img.shields.io/badge/license-personal%20free%20%7C%20commercial%20restricted-orange)](./LICENSE)

## Quick Start

Linux/macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex
```

Basic check:

```bash
nudge status
nudge info
```

Try completion in shell:

1. Type a partial command, e.g. `git st`
2. Press `Ctrl+E` to apply a suggestion
3. In Bash, press `Alt+/` to open multi-candidate popup

## Why Nudge

- Reduce repeated typing for common CLI workflows
- Keep suggestions aware of current project state
- Add risk checks before applying dangerous commands

## Capabilities

- LLM completion with context from history, cwd, and plugins
- Project-aware context: Git, Node.js, Python, Rust, Docker
- Safety warnings for dangerous commands
- Error diagnosis for failed commands (Zsh / PowerShell)
- Multi-shell support: Zsh, Bash, PowerShell, CMD

## Boundaries

- Nudge suggests commands; it does not execute them for you
- Auto ghost-text mode is only available in Zsh
- Bash/PowerShell/CMD use manual trigger mode (`Ctrl+E`)
- Bash popup can show multiple candidates; other shells currently use single-candidate apply path

## Usage

Common keys:

- `Ctrl+E`: manual completion (all shells, fastest baseline path)
- `Alt+/`: Bash popup selector
- `Tab`: accept auto suggestion in Zsh auto mode
- `Ctrl+G`: accept overlay suggestion when Zsh autosuggestions owns ghost text
- `F1`: toggle Zsh explanation details

Core commands:

```bash
nudge start
nudge stop
nudge restart
nudge status
nudge info
nudge doctor zsh
nudge doctor bash
```

## Installation Options

- One-click scripts: see [docs/installation.md](docs/installation.md)
- Source build: `cargo build --release`
- Shell integration refresh: `nudge setup <bash|zsh|powershell> --force`

## Configuration

Minimal local model example (`~/.nudge/config/config.yaml`):

```yaml
model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"

trigger:
  mode: manual
```

Remote API example:

```yaml
model:
  endpoint: "https://api.openai.com/v1"
  model_name: "gpt-4o-mini"
  api_key_env: "OPENAI_API_KEY"
```

More options: [docs/configuration.md](docs/configuration.md)

## Platform and Shell Matrix

| Shell | Manual (`Ctrl+E`) | Auto | Diagnosis | Notes |
|---|---|---|---|---|
| Zsh | Yes | Yes | Yes | Best full-feature experience |
| Bash | Yes | No | Planned | Popup selector on `Alt+/` |
| PowerShell 7.2+ | Yes | No | Yes | Via integration script/predictor |
| CMD | Yes | No | No | Basic integration only |

## Documentation

- [Installation Guide](docs/installation.md)
- [Configuration Reference](docs/configuration.md)
- [CLI Reference](docs/cli-reference.md)
- [Auto Mode Guide](docs/auto-mode.md)
- [Shell Guides](docs/shells/README.md)
- [FFI API](docs/ffi-api.md)

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## License

Free for personal/non-commercial use. Commercial use is restricted and requires separate permission. See [LICENSE](./LICENSE).
