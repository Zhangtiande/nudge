# Nudge

> A gentle nudge for your shell - LLM-powered CLI auto-completion

[English](./README.md) | [中文](./README_zh.md)

[![CI](https://github.com/Zhangtiande/nudge/actions/workflows/ci.yml/badge.svg)](https://github.com/Zhangtiande/nudge/actions/workflows/ci.yml)
[![Release](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml/badge.svg)](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/Zhangtiande/nudge)](https://github.com/Zhangtiande/nudge/releases/latest)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

---

Nudge uses Large Language Models to predict and complete command-line inputs based on your shell history, current directory context, and Git repository state.

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🤖 **AI-Powered Completions** | Uses LLM to understand context and suggest relevant commands |
| 📝 **History-Aware** | Learns from your shell history to provide personalized suggestions |
| 🔍 **Similar Command Search** | Automatically finds similar commands from history (like Bash Ctrl+R) |
| 🖥️ **System-Aware** | Adapts suggestions based on your OS, architecture, and shell type |
| 📁 **Context-Aware** | Considers current directory files and Git status |
| 🔒 **Privacy-First** | Automatically sanitizes sensitive data (API keys, passwords) before sending to LLM |
| ⚠️ **Safety Warnings** | Flags potentially dangerous commands (rm -rf, mkfs, etc.) |
| 🐚 **Multi-Shell Support** | Works with Bash, Zsh, PowerShell, and CMD |
| 🌐 **Cross-Platform** | Supports Linux, macOS, and Windows |
| ⚡ **Fast** | <200ms response time with local LLMs |
| 👻 **Auto Mode** | Ghost text suggestions as you type (like GitHub Copilot) |

## 🎬 Demo

**Zsh Auto Mode** - Ghost text suggestions appear as you type:

https://github.com/user-attachments/assets/2a625752-d047-4688-9252-3dc9c1cebed4

## 📋 Prerequisites

- **Rust** (for building from source)
- **Ollama** (for local LLM inference) or OpenAI API access

## 🖥️ Platform Support

Nudge provides pre-built binaries for multiple platforms. The build status and available downloads can be found on the [latest release](https://github.com/Zhangtiande/nudge/releases/latest) page.

> **Build Status**: [![Release](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml/badge.svg)](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml)
> Check the [Actions](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml) page for detailed build status of each platform.

| Platform | Architecture | Binary | Download |
|----------|--------------|--------|----------|
| **Linux** | x86_64 (glibc) | `nudge-linux-x86_64.tar.gz` | [📥 Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-x86_64.tar.gz) |
| **Linux** | x86_64 (musl) | `nudge-linux-x86_64-musl.tar.gz` | [📥 Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-x86_64-musl.tar.gz) |
| **Linux** | aarch64 (ARM64) | `nudge-linux-aarch64.tar.gz` | [📥 Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-aarch64.tar.gz) |
| **macOS** | x86_64 (Intel) | `nudge-macos-x86_64.tar.gz` | [📥 Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-macos-x86_64.tar.gz) |
| **macOS** | aarch64 (Apple Silicon) | `nudge-macos-aarch64.tar.gz` | [📥 Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-macos-aarch64.tar.gz) |
| **Windows** | x86_64 | `nudge-windows-x86_64.zip` | [📥 Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-windows-x86_64.zip) |

> **Note**: Download links will only work after a successful release build. If a platform's build fails, its binary will not be available in the release.

### Shell Support

| Shell | Linux | macOS | Windows | Auto Mode | Integration |
|-------|-------|-------|---------|-----------|-------------|
| Bash | ✅ | ✅ | ✅ (WSL/Git Bash) | 🚧 (Planned) | `integration.bash` |
| Zsh | ✅ | ✅ | ✅ (WSL) | ✅ (POSTDISPLAY) | `integration.zsh` |
| PowerShell 7.2+ | ❌ | ❌ | ✅ | 🚧 (Planned) | `integration.ps1` |
| PowerShell 5.1 | ❌ | ❌ | ✅ | ❌ (Manual only) | `integration.ps1` |
| CMD | ❌ | ❌ | ✅ | ❌ (Manual only) | `integration.cmd` |

> **Note**: Auto Mode is currently only fully supported in **Zsh**. Support for Bash and PowerShell is planned.

## 📦 Installation

### Quick Install

**Unix/Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex
```

The installer downloads the binary, adds it to PATH, configures shell integration, and starts the daemon.

For manual installation, custom options, or building from source, see the [Installation Guide](docs/installation.md).

## 🚀 Usage

### Quick Start

After installation, the daemon should be running automatically. Simply press `Ctrl+E` while typing a command to trigger completion.

### Trigger Modes

Nudge supports two trigger modes:

**Manual Mode** (default): Press `Ctrl+E` to trigger completion on demand.

**Auto Mode**: Suggestions appear automatically as you type, displayed as ghost text (gray text after your cursor).

```yaml
# Enable auto mode in config.yaml
trigger:
  mode: auto              # "manual" or "auto"
  auto_delay_ms: 500      # Debounce delay before triggering
```

| Key | Action |
|-----|--------|
| `Ctrl+E` | Trigger completion (both modes) |
| `Tab` | Accept full suggestion (auto mode) |
| `Right Arrow` | Accept next word (Zsh/PowerShell) |

For detailed auto mode documentation, see [Auto Mode Guide](docs/auto-mode.md).

If you need to manually configure shell integration:

```bash
nudge setup
```

Then restart your shell or source your profile:

```bash
# Bash
source ~/.bashrc

# Zsh
source ~/.zshrc

# PowerShell
. $PROFILE
```

### Common Commands

```bash
# Start daemon
nudge start

# Check daemon status
nudge status

# Stop daemon
nudge stop

# Restart daemon (after config changes)
nudge restart

# Show runtime information
nudge info

# Show runtime information as JSON
nudge info --json

# Get specific field (useful in scripts)
nudge info --field config_dir
```

For a complete CLI reference, see [CLI Reference](docs/cli-reference.md).

## ⚙️ Configuration

For detailed configuration options, see the [Configuration Reference](docs/configuration.md).

**Quick start example** (`~/.config/nudge/config.yaml` on Linux/macOS, `%APPDATA%\nudge\config\config.yaml` on Windows):

```yaml
# Model Configuration
model:
  endpoint: "http://localhost:11434/v1"  # Ollama default
  model_name: "codellama:7b"
  timeout_ms: 5000

# Context Settings
context:
  history_window: 20              # Recent command history
  include_cwd_listing: true       # Current directory files
  include_system_info: true       # OS, architecture, shell, user
  similar_commands_enabled: true  # Search similar commands (like Ctrl+R)
  similar_commands_window: 200    # Search last 200 history entries
  similar_commands_max: 5         # Return up to 5 similar commands
  max_files_in_listing: 50
  max_total_tokens: 4000

# Trigger Settings
trigger:
  mode: "manual"            # "manual" or "auto"
  hotkey: "\C-e"            # Ctrl+E
  auto_delay_ms: 500        # Debounce delay for auto mode

# Git Plugin
plugins:
  git:
    enabled: true
    depth: standard  # light, standard, or detailed

# Privacy
privacy:
  sanitize_enabled: true
  block_dangerous: true

# Logging
log:
  level: "info"
  file_enabled: false  # Enable for daily-rotated file logs
```

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Nudge Binary                               │
├─────────────────────────────┬───────────────────────────────────────┤
│         Client Mode         │            Daemon Mode                │
├─────────────────────────────┼───────────────────────────────────────┤
│  • Capture buffer/cursor    │  • IPC Server                         │
│  • Send request via IPC     │    ├─ Unix: Unix Domain Socket        │
│  • Output completion        │    └─ Windows: Named Pipe             │
│                             │  • Context Engine                     │
│                             │    ├─ History Reader                  │
│                             │    ├─ CWD Scanner                     │
│                             │    └─ Git Plugin                      │
│                             │  • LLM Connector                      │
│                             │  • Sanitizer                          │
└─────────────────────────────┴───────────────────────────────────────┘
```

**How it works:**

1. Shell hook captures input buffer on hotkey press
2. Client sends request to daemon via IPC (Unix socket or Named Pipe)
3. Daemon gathers context (history, CWD files, Git status)
4. Sanitizer removes sensitive data
5. LLM generates completion
6. Safety check flags dangerous commands
7. Client outputs suggestion to shell

## 🔌 LLM Providers

### Local (Ollama)

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Pull a model
ollama pull codellama:7b

# Start Ollama server
ollama serve
```

### OpenAI / Compatible APIs

```yaml
# ~/.config/nudge/config.yaml
model:
  endpoint: "https://api.openai.com/v1"
  model_name: "gpt-3.5-turbo"
  api_key_env: "OPENAI_API_KEY"
```

```bash
export OPENAI_API_KEY="sk-..."
```

### Alibaba DashScope (Qwen)

```yaml
model:
  endpoint: "https://dashscope.aliyuncs.com/compatible-mode/v1"
  model_name: "qwen3-coder-flash"
  api_key_env: "DASHSCOPE_API_KEY"
```

## 🛠️ Development

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- daemon --foreground

# Check code
cargo clippy

# Format code
cargo fmt
```

## 🗺️ Roadmap

Nudge is actively evolving with exciting features planned. Here's a glimpse of what's coming:

### 🎯 Upcoming Features

| Feature | Description | Status |
|---------|-------------|--------|
| **Project-Aware Context** | Auto-activate plugins based on command keywords (docker, npm, etc.) to provide deep project context | 🎯 Planned |
| **Error Context Recovery** | Automatically collect error context and provide intelligent fix suggestions when commands fail | 🎯 Planned |
| **Smart History Analytics** | Analyze command patterns and suggest aliases for frequently used commands | 🎯 Planned |
| **Community Plugin System** | WASM-based plugin marketplace for custom context providers | 🎯 Planned |

### 🔌 Planned Plugins

Expanding beyond Git to provide context for:
- **Docker**: Dockerfile, compose files, running containers
- **Node.js**: package.json, scripts, dependencies
- **Python**: requirements.txt, virtual environments, pip packages
- **Rust**: Cargo.toml, workspace info
- **Kubernetes**: kubectl context, pods, resources
- **Terraform**: .tf files, workspaces, state
- **Databases**: connection configs, schemas

**📖 Full Roadmap**: See [ROADMAP.md](./ROADMAP.md) for detailed feature specifications, technical implementation plans, and release timeline.

## 📄 License

MIT

## 🤝 Contributing

Contributions are welcome! Please open an issue or pull request.
