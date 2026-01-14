# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Nudge** is an LLM-powered CLI auto-completion tool written in Rust. It provides intelligent command suggestions for Bash, Zsh, PowerShell, and CMD shells by leveraging LLMs (Ollama, OpenAI, or any OpenAI-compatible API).

Key features:
- AI-powered command completion based on shell history, current directory, and Git state
- Privacy-first design with automatic sanitization of sensitive data (API keys, passwords)
- Safety warnings for dangerous commands (rm -rf, fork bombs, etc.)
- Daemon-based architecture for low-latency IPC via Unix Domain Sockets (Unix) or Named Pipes (Windows)
- Cross-platform support: Linux, macOS, and Windows

## Build & Test Commands

```bash
# Debug build
cargo build

# Release build (recommended)
cargo build --release

# Run all tests
cargo test

# Run specific test module
cargo test sanitizer      # Sanitizer tests
cargo test git_plugin     # Git plugin tests
cargo test completion     # Integration tests

# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run clippy lints
cargo clippy
```

## Architecture

```
Shell (Bash/Zsh/PowerShell/CMD) → nudge complete (CLI) → IPC → nudge daemon (server)
                                                          │
                                    ┌─────────────────────┴─────────────────────┐
                                    │   Unix: Unix Domain Socket                │
                                    │   Windows: Named Pipe (\\.\pipe\nudge_*)  │
                                    └─────────────────────┬─────────────────────┘
                                                          ↓
                              ┌───────────────────────────┘
                              ↓
                    ┌─────────┴─────────┬───────────────────────┐
                    ↓                   ↓                       ↓
              Context Engine      Sanitizer           LLM Connector
              - History Reader    - Regex patterns     - OpenAI-compatible
              - CWD Scanner       - Custom patterns    - Response parsing
              - Git Plugin        - API key removal    - Safety checker
```

### Key Modules

- **`src/cli.rs`**: CLI argument definitions using clap (daemon, complete, start, stop, status)
- **`src/config.rs`**: YAML configuration loading with env var override support
- **`src/client/ipc.rs`**: Unix socket client with liveness check and timeouts
- **`src/daemon/server.rs`**: IPC server handling concurrent requests
- **`src/daemon/llm.rs`**: LLM API client with prompt building
- **`src/daemon/context/`**: Context aggregation with priority-based truncation
- **`src/daemon/sanitizer.rs`**: Sensitive data redaction (API keys, tokens, passwords)
- **`src/daemon/safety.rs`**: Dangerous command detection

### Data Flow

1. Shell hook captures buffer/cursor on Ctrl+E
2. Client sends `CompletionRequest` via IPC
3. Server gathers context (history, cwd, git) → sanitizes → builds prompt → calls LLM
4. Server checks completion for dangerous commands → returns `CompletionResponse`
5. Client outputs suggestion text or JSON

### Critical Implementation Details

- **Async throughout**: Uses Tokio for all I/O operations
- **Session tracking**: Maintains shell session continuity via session_id
- **Git plugin timeout**: Strict 50ms timeout on all git operations
- **Token estimation**: Word-based with 1.3x multiplier for context truncation
- **Context priorities**: history(80) > cwd_listing(60) > plugins(40)

### Platform-Specific Details

#### Unix (Linux/macOS)
- **IPC**: Unix Domain Socket at `~/.config/nudge/nudge.sock`
- **Process control**: Uses `nix` crate with POSIX signals (SIGTERM, SIGCONT)
- **Shell integration**: Bash (`integration.bash`) and Zsh (`integration.zsh`)

#### Windows
- **IPC**: Named Pipe at `\\.\pipe\nudge_{username}`
- **Process control**: Uses `windows-sys` crate with `OpenProcess`/`TerminateProcess`
- **Shell integration**: PowerShell (`integration.ps1`) and CMD (`integration.cmd`)
- **Installation**: Run `shell/install.ps1` to add to PowerShell profile
