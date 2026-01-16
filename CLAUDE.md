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

## Pre-Commit Quality Checks

**IMPORTANT**: Before any code changes are committed, you MUST run the following checks to ensure code quality and prevent CI failures. These checks mirror the GitHub Actions workflows defined in `.github/workflows/`.

### Required Checks (Run in sequence)

```bash
# 1. Format check - Ensures code follows Rust style guidelines
cargo fmt --all -- --check

# 2. Clippy check - Catches common mistakes and enforces best practices
cargo clippy --all-targets --all-features -- -D warnings

# 3. Compile check - Verifies code compiles on current platform
cargo check --all-targets

# 4. Test suite - Ensures all tests pass
cargo test --verbose
```

### When to Run These Checks

- **Always**: After making ANY code changes (fixes, features, refactors)
- **Before**: Asking the user to commit or create a pull request
- **After**: Resolving merge conflicts or updating dependencies

### Handling Check Failures

If any check fails:
1. **DO NOT** proceed with commit/push
2. Fix the issues reported by the failing check
3. Re-run all checks from the beginning
4. Only proceed when ALL checks pass

### Quick Check (Faster alternative for iterative development)

For rapid iteration during development, you can use:
```bash
# Fast validation (skips tests)
cargo check --all-targets && cargo clippy --all-targets -- -D warnings
```

But always run the full check suite before final commit.

### Cross-Platform Considerations

- The CI runs on **Ubuntu, macOS, and Windows**
- If you make platform-specific changes (e.g., `#[cfg(windows)]`), note this in your commit message
- Consider running checks on multiple platforms when possible (though not required locally)

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
