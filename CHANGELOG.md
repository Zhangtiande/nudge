# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2026-02-07

### Added
- Bash popup selector now supports multiple LLM-generated candidates with list-only navigation.
- Shell-mode completion responses now include concise `summary_short` and `reason_short` metadata for UI display.

### Changed
- Refactored completion prompts into dedicated shell-mode template files for maintainability and mode-specific behavior.
- Preserved `Ctrl+E` manual completion as the fastest single-candidate fallback path across all shells.
- Reorganized documentation into per-shell guides with explicit mode boundaries and quick-start-first structure.
- Updated license policy to personal-use-free with commercial restrictions.

### Fixed
- Zsh auto overlay now preserves list metadata parsing when optional fields are empty.
- Improved Zsh overlay readability by reducing low-signal `why` text and delimiter ambiguity in compact rendering.
- Added daemon-side logging for raw and parsed LLM completion payloads to simplify integration debugging.

## [0.4.5] - 2026-02-07

### Breaking Changes
- Unified all nudge runtime/config paths under `~/.nudge` (for example: `config/`, `run/`, `logs/`, `data/`, `modules/`, `shell/`, `lib/`).
- `nudge setup` now initializes fresh configuration under the new root; old path compatibility/migration is intentionally not provided.
- Removed deprecated configuration entrypoints and legacy shell setup scripts (`nudge config`, `shell/setup-shell.sh`, `shell/setup-shell.ps1`).

### Fixed
- In autosuggestions-owned ghost mode, `Ctrl+G` now accepts Nudge overlay suggestions and clears stale gray preview text.

## [0.4.4] - 2026-02-06

### Added
- **Zsh Ghost Ownership Strategy**: New `trigger.zsh_ghost_owner` mode selection for conflict-safe coexistence with `zsh-autosuggestions`
  - `auto`: Prefer `zsh-autosuggestions` when available
  - `nudge`: Force Nudge-owned ghost text
  - `autosuggestions`: Keep ghost text owned by `zsh-autosuggestions`, use overlay accept key
- **Overlay Backend Selection**: New `trigger.zsh_overlay_backend` with `message` and `rprompt`
- **`nudge doctor zsh`**: New diagnostics command for Zsh integration health, key bindings, hooks, and daemon latency sampling
- **Explanation Layer**: Overlay explanation toggle (`F1`) with `why/risk/diff` details
- **Zsh Integration Test Coverage**: Added dedicated tests for key bindings, overlay rendering, RPROMPT restore/reapply, and history navigation behavior

### Changed
- **Event-driven Auto Fetch**: Replaced sleep-based debounce path with event-driven request triggering and async generation arbitration
- **Mode-specific Overlay UX**:
  - `message` backend shows full overlay context (`why/risk/diff`)
  - `rprompt` backend shows compact `diff` only
- **Partial Accept Scope**: Kept stable `Right Arrow` word accept and removed unstable `Alt+Right` / `Ctrl+Right` bindings

### Fixed
- Overlay mode no longer clears `zsh-autosuggestions` ghost text (`POSTDISPLAY`)
- Up/Down history navigation no longer triggers overlay async fetch loops (reduces input lag)
- RPROMPT overlay now reliably reapplies after external prompt wipes
- Manual hotkey completion now clears stale inline suggestion display to avoid visual overlap
- Message backend overlay no longer leaks ANSI escape descriptors as raw text

## [0.4.3] - 2026-02-05

### Added
- **Danger Warning System**: Visual warnings for dangerous command suggestions
  - Zsh: Yellow `⚠️` prefix in ghost text for dangerous suggestions
  - Bash: Warning prefix in suggestion output
  - PowerShell: Warning display in PSReadLine predictor
  - CMD: Warning prefix documentation
  - Client emits `__NUDGE_WARNING__` sentinel for shell integration
- **Auto Mode Widget Refactor**: Complete rewrite of Zsh auto mode for better performance
  - Widget-based triggering instead of `zle-line-pre-redraw` hook
  - Widget classification: modify/clear/accept/partial_accept/ignore
  - Time-based debounce using `sleep` + `zle -F` (replaces per-keystroke spawning)
  - Async fetch with proper `$BUFFER` access via callback widgets
  - `$PENDING` detection to skip fetch when input queue is not empty

### Fixed
- Zsh auto mode no longer causes terminal freezing on rapid typing
- Arrow keys (history navigation) now properly clear suggestions without triggering new fetch
- Ghost text display now correctly handles warning prefix from dangerous suggestions

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

## [0.4.0] - 2026-02-03

### Added
- **Error Diagnosis**: AI-powered error analysis and fix suggestions
  - Captures stderr from failed commands automatically
  - Provides context-aware fix suggestions using LLM
  - Tab key to accept diagnosis suggestion in Zsh manual mode
  - Full project context (git, cwd, history) included in diagnosis
- **Shell Integration**:
  - Zsh: Diagnosis display with visual hint, Tab acceptance
  - PowerShell: Error diagnosis with Tab completion
  - Bash: Basic diagnosis support
- **CLI**: New `nudge diagnose` subcommand for manual diagnosis
- **Configuration**: New `diagnosis` section with `enabled`, `max_stderr_lines`, `interactive_commands` options

### Changed
- Setup command now installs config files during setup
- FFI layer updated to use `GatherParams` for context gathering

## [0.3.3] - 2026-02-01

### Added
- **Project-Aware Plugins**: Language-specific context for better suggestions
  - **Node.js Plugin**: Detects package.json, lock files, scripts, dependencies, monorepo structure
  - **Rust Plugin**: Parses Cargo.toml, workspace members, binary targets, rust-version
  - **Python Plugin**: Supports pyproject.toml (PEP 621), requirements.txt, uv/poetry lock files
- Plugin integration tests for all three language plugins

### Changed
- Removed auto mode from Bash integration (recommend Zsh for auto mode)
- Updated roadmap to mark project plugins as completed

## [0.3.2] - 2026-01-28

### Fixed
- PowerShell profile detection improved for setup command
- Console logging disabled for all CLI commands except foreground daemon
- Documentation clarified PowerShell auto mode limitation (PSReadLine timeout)

### Changed
- Config path handling updated with async setup functions
- GitHub Actions optimized with staged pipeline and smart caching

## [0.3.1] - 2026-01-25

### Fixed
- Daemon now creates new process group to prevent terminal signal interference
- Zsh auto mode enhanced with better functionality

### Added
- Local mode support for installation scripts
- Demo video for Zsh auto mode in README

### Changed
- Removed deprecated settings.local.json file
- Added .claude/ to .gitignore

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

[Unreleased]: https://github.com/Zhangtiande/nudge/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/Zhangtiande/nudge/compare/v0.4.5...v0.5.0
[0.4.5]: https://github.com/Zhangtiande/nudge/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/Zhangtiande/nudge/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/Zhangtiande/nudge/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/Zhangtiande/nudge/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/Zhangtiande/nudge/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/Zhangtiande/nudge/compare/v0.3.3...v0.4.0
[0.3.3]: https://github.com/Zhangtiande/nudge/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/Zhangtiande/nudge/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/Zhangtiande/nudge/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/Zhangtiande/nudge/compare/v0.2.3...v0.3.0
[0.2.3]: https://github.com/Zhangtiande/nudge/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/Zhangtiande/nudge/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/Zhangtiande/nudge/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/Zhangtiande/nudge/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Zhangtiande/nudge/releases/tag/v0.1.0
