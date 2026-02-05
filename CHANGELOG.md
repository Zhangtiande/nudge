# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.2] - 2026-02-05

### Added
- **Suggestion Cache**: LRU+TTL cache with stale-while-revalidate for faster completions
  - Cache key: `prefix + cwd + git_state + shell_mode` (context-aware)
  - TTL: auto=5min, manual=10min, negative=30s
  - Stale-while-revalidate: returns cached result immediately, refreshes in background at 80% TTL
- **Cache Debug Logging**: Run `RUST_LOG=debug nudge daemon` to observe cache hit/miss

### Changed
- Removed `time_bucket` from cache key (unnecessary with debounce delay)
- Extended cache TTL for better reuse across sessions

### Documentation
- Added auto-mode widget refactor implementation plan
- Updated CLAUDE.md with cache design and Zsh auto mode architecture

## [0.4.1] - 2026-02-04

### Added
- **Interactive Command Detection**: Shell integration now detects interactive shells to avoid breaking scp, rsync, git remote operations
  - Bash/Zsh: `[[ $- == *i* ]]` check
  - PowerShell: `[Environment]::UserInteractive` check
  - CMD: `if defined PROMPT` check
- **Interactive Commands Config**: New `diagnosis.interactive_commands` setting to skip stderr capture for programs like vim, ssh, top, less, fzf, tmux, python, etc.
- **CLI Enhancement**: `nudge info --field interactive_commands` to expose config to shell scripts

### Fixed
- Shell integration scripts no longer output messages in non-interactive shells
- Error diagnosis now skips stderr capture for interactive programs
- **Security**: Updated `bytes` crate to 1.11.1 to fix integer overflow vulnerability (RUSTSEC-2026-0007)

## [0.3.0] - 2026-01-23

### Added
- **Auto Mode**: Ghost text suggestions as you type (like GitHub Copilot)
  - Zsh: Native POSTDISPLAY integration
  - Bash: ANSI escape code preview
  - PowerShell 7.2+: PSReadLine predictor API
- **FFI Layer** (Unix only): Dynamic library for lower latency
  - `libnudge.so` (Linux) / `libnudge.dylib` (macOS)
  - Direct function calls instead of CLI invocation
  - Embedded Tokio runtime for async operations
- **NudgePredictor PowerShell Module**: Native PSReadLine integration
  - Throttling and caching for optimal performance
  - Automatic registration with SubsystemManager
- **New Documentation**:
  - Auto mode guide (`docs/auto-mode.md`)
  - Migration guide (`docs/migration-guide.md`)
  - Troubleshooting guide (`docs/troubleshooting.md`)
  - Installation guide (`docs/installation.md`)
  - FFI API documentation (`docs/ffi-api.md`)
  - Release notes (`docs/RELEASE_NOTES_v0.3.0.md`)

### Changed
- Shell integration scripts updated for auto mode support
- Installation scripts now install NudgePredictor module on Windows
- README simplified with link to detailed installation guide
- CI workflow updated with FFI feature tests

### Configuration
New trigger options:
```yaml
trigger:
  mode: "auto"          # "manual" or "auto"
  auto_delay_ms: 500    # Debounce delay for auto mode
```

## [0.2.3] - 2026-01-20

### Added
- Plugin system with Git and Docker plugins
- `nudge info` command with JSON output
- `nudge setup` command for shell integration
- Centralized platform detection in Rust

### Changed
- Shell integration scripts now use `nudge info` for paths
- Installation scripts use `nudge setup` for configuration

## [0.2.2] - 2026-01-19

### Added
- Layered configuration system (default + user config)
- Similar command search (like Ctrl+R)
- System info context (OS, architecture, shell, user)

### Changed
- Configuration file organization improved
- Better error messages for config loading

## [0.2.1] - 2026-01-18

### Added
- Windows support (PowerShell and CMD)
- Named Pipe IPC for Windows
- Cross-platform path handling

### Fixed
- Various Windows compatibility issues

## [0.2.0] - 2026-01-16

### Added
- Cross-platform refactoring
- Windows Named Pipe support
- PowerShell integration script
- CMD integration script

### Changed
- IPC abstraction for Unix/Windows
- Platform-specific path handling

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

[Unreleased]: https://github.com/Zhangtiande/nudge/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/Zhangtiande/nudge/compare/v0.2.3...v0.3.0
[0.2.3]: https://github.com/Zhangtiande/nudge/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/Zhangtiande/nudge/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/Zhangtiande/nudge/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/Zhangtiande/nudge/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Zhangtiande/nudge/releases/tag/v0.1.0
