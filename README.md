# Nudge

> A gentle nudge for your shell - LLM-powered CLI auto-completion

[English](./README.md) | [中文](./README_zh.md)

[![CI](https://github.com/Zhangtiande/nudge/actions/workflows/ci.yml/badge.svg)](https://github.com/Zhangtiande/nudge/actions/workflows/ci.yml)
[![Release](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml/badge.svg)](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/Zhangtiande/nudge)](https://github.com/Zhangtiande/nudge/releases/latest)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

Nudge uses Large Language Models to predict and complete command-line inputs based on your shell history, current directory context, and project state.

## Features

- **AI-Powered Completions** - Uses LLM to understand context and suggest relevant commands
- **Project-Aware** - Automatically detects Git, Node.js, Python, Rust, Docker projects and provides deep context
- **History-Aware** - Learns from your shell history with similar command search (like Ctrl+R)
- **System-Aware** - Adapts suggestions based on your OS, architecture, and shell type
- **Error Diagnosis** - Automatically analyzes failed commands and suggests fixes
- **Privacy-First** - Sanitizes sensitive data (API keys, passwords) before sending to LLM
- **Safety Warnings** - Flags potentially dangerous commands (rm -rf, mkfs, etc.)
- **Multi-Shell** - Works with Bash, Zsh, PowerShell, and CMD
- **Cross-Platform** - Supports Linux, macOS, and Windows
- **Fast** - <200ms response time with local LLMs
- **Auto Mode** - Ghost text suggestions as you type (Zsh only)

## Demo

**Zsh Auto Mode** - Ghost text suggestions appear as you type:

https://github.com/user-attachments/assets/766247e1-1cf2-47da-96e7-045415ede013

## Quick Start

### Installation

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex
```

The installer downloads the binary, configures shell integration, and starts the daemon.

For manual installation or building from source, see the [Installation Guide](docs/installation.md).

### Basic Usage

After installation, press `Ctrl+E` while typing a command to trigger completion.

```bash
# Start daemon (if not auto-started)
nudge start

# Check status
nudge status

# Show runtime info
nudge info
```

### Configuration

Create `~/.config/nudge/config.yaml` (Linux/macOS) or `%APPDATA%\nudge\config\config.yaml` (Windows):

```yaml
model:
  endpoint: "http://localhost:11434/v1"  # Ollama default
  model_name: "codellama:7b"

trigger:
  mode: "manual"        # "manual" or "auto"
  auto_delay_ms: 500    # Debounce for auto mode

diagnosis:
  enabled: true         # Enable error diagnosis
```

See [Configuration Reference](docs/configuration.md) for all options.

## Trigger Modes

| Mode | Description | Supported Shells |
|------|-------------|------------------|
| **Manual** | Press `Ctrl+E` to trigger | All shells |
| **Auto** | Ghost text appears as you type | Zsh only |

| Key | Action |
|-----|--------|
| `Ctrl+E` | Trigger completion |
| `Tab` | Accept suggestion (auto mode) |
| `Right Arrow` | Accept next word (Zsh) |

## Error Diagnosis

When a command fails, Nudge analyzes the error with full project context and suggests a fix.

**Zsh:**
```
$ gti status
zsh: command not found: gti
❌ Typo: 'gti' should be 'git'

git status          ← Tab to accept
```

**PowerShell:**
```
PS> gti status
[Error] Command not found: 'gti'
[Tip] Typo: did you mean 'git'?

PS> █               ← Tab to accept
```

Enable in config:
```yaml
diagnosis:
  enabled: true
```

> [!CAUTION]
> When error diagnosis is enabled, stderr is temporarily captured during command execution. This means progress output from tools like `cargo build`, `npm install`, or `docker pull` will appear **after** the command completes rather than in real-time. If you need real-time stderr output, disable diagnosis with `diagnosis.enabled: false`.

## Project-Aware Context

Nudge automatically detects your project type and provides relevant context to the LLM:

| Project Type | Detection | Context Provided |
|--------------|-----------|------------------|
| **Git** | `.git` directory | Branch, staged files, recent commits |
| **Node.js** | `package.json` | Scripts, dependencies, package manager |
| **Python** | `pyproject.toml`, `requirements.txt` | Dependencies, virtual env, Python version |
| **Rust** | `Cargo.toml` | Dependencies, targets, workspace info |
| **Docker** | `Dockerfile`, `compose.yaml` | Services, images, running containers |

## LLM Providers

### Ollama (Local)

```bash
ollama pull codellama:7b
ollama serve
```

```yaml
model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"
```

### OpenAI

```yaml
model:
  endpoint: "https://api.openai.com/v1"
  model_name: "gpt-4o-mini"
  api_key_env: "OPENAI_API_KEY"
```

### Alibaba DashScope (Qwen)

```yaml
model:
  endpoint: "https://dashscope.aliyuncs.com/compatible-mode/v1"
  model_name: "qwen-coder-plus"
  api_key_env: "DASHSCOPE_API_KEY"
```

## Platform Support

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x86_64 (glibc) | [Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-x86_64.tar.gz) |
| Linux | x86_64 (musl) | [Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-x86_64-musl.tar.gz) |
| Linux | aarch64 | [Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-aarch64.tar.gz) |
| macOS | x86_64 (Intel) | [Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-macos-x86_64.tar.gz) |
| macOS | aarch64 (Apple Silicon) | [Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-macos-aarch64.tar.gz) |
| Windows | x86_64 | [Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-windows-x86_64.zip) |

### Shell Support

| Shell | Manual Mode | Auto Mode | Error Diagnosis |
|-------|-------------|-----------|-----------------|
| Zsh | ✅ | ✅ | ✅ |
| Bash | ✅ | ❌ | Planned |
| PowerShell 7.2+ | ✅ | ❌ | ✅ |
| PowerShell 5.1 | ✅ | ❌ | ✅ |
| CMD | ✅ | ❌ | ❌ |

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Nudge Binary                               │
├─────────────────────────────┬───────────────────────────────────────┤
│         Client Mode         │            Daemon Mode                │
├─────────────────────────────┼───────────────────────────────────────┤
│  • Capture buffer/cursor    │  • IPC Server (Socket/Named Pipe)     │
│  • Send request via IPC     │  • Context Engine                     │
│  • Output completion        │    ├─ History, CWD, System Info       │
│                             │    └─ Plugins (Git, Node, Python...)  │
│                             │  • LLM Connector                      │
│                             │  • Sanitizer & Safety Checker         │
└─────────────────────────────┴───────────────────────────────────────┘
```

## Documentation

- [Installation Guide](docs/installation.md)
- [Configuration Reference](docs/configuration.md)
- [CLI Reference](docs/cli-reference.md)
- [Auto Mode Guide](docs/auto-mode.md)
- [Roadmap](ROADMAP.md)

## Development

```bash
cargo build --release
cargo test
cargo clippy
cargo fmt
```

## Contributing

Contributions are welcome! Please open an issue or pull request.

## License

MIT
