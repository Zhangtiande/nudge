# Nudge

> A gentle nudge for your shell - LLM-powered CLI auto-completion

[English](./README.md) | [中文](./README_zh.md)

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
| 🐚 **Multi-Shell Support** | Works with Bash and Zsh |
| ⚡ **Fast** | <200ms response time with local LLMs |

## 📋 Prerequisites

- **Rust** (for building from source)
- **Ollama** (for local LLM inference) or OpenAI API access

## 📦 Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/user/nudge.git
cd nudge

# Build release binary
cargo build --release

# Install to /usr/local/bin
sudo cp target/release/nudge /usr/local/bin/

# Run the installer script
./shell/install.sh
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
```

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Nudge Binary                               │
├─────────────────────────────┬───────────────────────────────────────┤
│         Client Mode         │            Daemon Mode                │
├─────────────────────────────┼───────────────────────────────────────┤
│  • Capture buffer/cursor    │  • IPC Server (Unix Socket)           │
│  • Send request via IPC     │  • Context Engine                     │
│  • Output completion        │    ├─ History Reader                  │
│                             │    ├─ CWD Scanner                     │
│                             │    └─ Git Plugin                      │
│                             │  • LLM Connector                      │
│                             │  • Sanitizer                          │
└─────────────────────────────┴───────────────────────────────────────┘
```

**How it works:**

1. Shell hook captures input buffer on hotkey press
2. Client sends request to daemon via Unix socket
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
