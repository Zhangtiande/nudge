# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Integration tests for completion flow, sanitizer, and git plugin
- Test fixtures for bash and zsh history files
- Configuration documentation

## [0.1.0] - 2026-01-13

### Added
- Initial release of nudge - LLM-powered CLI auto-completion
- Context-aware completion using shell history, CWD, and git context
- Sensitive data protection with regex-based sanitization
- Dangerous command warning system
- Shell integration for Bash and Zsh
- Daemon mode with Unix Domain Socket IPC
- Configuration via YAML file

### Core Features
- **Context Engine**
  - Shell history reading (bash and zsh formats)
  - Current working directory file listing
  - Git plugin with three depth levels (light/standard/detailed)
  - Priority-based context truncation

- **Privacy & Safety**
  - Built-in patterns for API keys, tokens, and passwords
  - Custom sanitization pattern support
  - Dangerous command detection (rm -rf, fork bombs, etc.)
  - Warning system for destructive operations

- **Shell Integration**
  - Bash integration via `bind -x`
  - Zsh integration via ZLE widget
  - Lazy daemon startup with flock-based concurrency control
  - Ctrl+E hotkey binding (configurable)

- **LLM Integration**
  - OpenAI-compatible API client
  - Support for local models (Ollama) and cloud services
  - Configurable timeout and model selection

### Technical Details
- Single binary distribution (client + daemon in one executable)
- Async runtime with Tokio
- Unix Domain Socket for low-latency IPC
- JSON-based request/response protocol
- 50ms strict timeout for git operations
- Word-based token estimation for context truncation

### Configuration
- YAML configuration file at `~/.config/nudge/config.yaml`
- Environment variable override via `SMARTSHELL_CONFIG`
- Sensible defaults for all settings

### Documentation
- README with installation instructions (English)
- README_zh.md with Chinese documentation
- Configuration reference documentation

[Unreleased]: https://github.com/user/nudge/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/user/nudge/releases/tag/v0.1.0
