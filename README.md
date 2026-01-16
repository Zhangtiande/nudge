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
| 📁 **Context-Aware** | Considers current directory files and Git status |
| 🔒 **Privacy-First** | Automatically sanitizes sensitive data (API keys, passwords) before sending to LLM |
| ⚠️ **Safety Warnings** | Flags potentially dangerous commands (rm -rf, mkfs, etc.) |
| 🐚 **Multi-Shell Support** | Works with Bash, Zsh, PowerShell, and CMD |
| 🖥️ **Cross-Platform** | Supports Linux, macOS, and Windows |
| ⚡ **Fast** | <200ms response time with local LLMs |

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

| Shell | Linux | macOS | Windows | Integration |
|-------|-------|-------|---------|-------------|
| Bash | ✅ | ✅ | ✅ (WSL/Git Bash) | `integration.bash` |
| Zsh | ✅ | ✅ | ✅ (WSL) | `integration.zsh` |
| PowerShell | ❌ | ❌ | ✅ | `integration.ps1` |
| CMD | ❌ | ❌ | ✅ | `integration.cmd` |

## 📦 Installation

### From Pre-built Binaries (Recommended)

Download the latest release for your platform from the [Releases page](https://github.com/Zhangtiande/nudge/releases/latest).

**Linux/macOS:**
```bash
# Download and extract (replace with your platform's binary)
curl -L https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-x86_64.tar.gz | tar xz

# Move to PATH
sudo mv nudge /usr/local/bin/

# Run the installer
nudge daemon --install
```

**Windows (PowerShell):**
```powershell
# Download from releases page and extract
# Then add to PATH and run:
.\nudge.exe daemon --install
```

### From Source

```bash
# Clone the repository
git clone https://github.com/user/nudge.git
cd nudge

# Build release binary
cargo build --release

# Install (Unix)
sudo cp target/release/nudge /usr/local/bin/
./shell/install.sh

# Install (Windows PowerShell, run as Administrator)
# Copy target\release\nudge.exe to a directory in your PATH
# Then run the installer:
# .\shell\install.ps1
```

### Quick Setup

After installation, add to your shell RC file:

**Bash** (`~/.bashrc`):
```bash
[ -f "$HOME/.config/nudge/integration.bash" ] && source "$HOME/.config/nudge/integration.bash"
```

**Zsh** (`~/.zshrc`):
```zsh
[ -f "$HOME/.config/nudge/integration.zsh" ] && source "$HOME/.config/nudge/integration.zsh"
```

**PowerShell** (automatic via `install.ps1`, or manually add to `$PROFILE`):
```powershell
. "C:\path\to\integration.ps1"
```

**CMD** (run `integration.cmd` or add to AutoRun registry):
```cmd
"C:\path\to\integration.cmd"
```

## 🚀 Usage

1. **Start the Daemon** (automatic with lazy-loading, or manually):
   ```bash
   nudge daemon --fork
   ```

2. **Trigger Completion**: Press `Ctrl+E` while typing a command

3. **Check Status**:
   ```bash
   nudge status
   ```

4. **Stop Daemon**:
   ```bash
   nudge daemon stop
   ```

## ⚙️ Configuration

Configuration file: `~/.config/nudge/config.yaml`

```yaml
# Model Configuration
model:
  endpoint: "http://localhost:11434/v1"  # Ollama default
  model_name: "codellama:7b"
  timeout_ms: 5000

# Context Settings
context:
  history_window: 20
  include_cwd_listing: true
  max_files_in_listing: 50
  max_total_tokens: 4000

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

## 📄 License

MIT

## 🤝 Contributing

Contributions are welcome! Please open an issue or pull request.
