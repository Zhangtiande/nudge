# Nudge Roadmap

[English](roadmap.md) | [中文](../zh/roadmap.md)

This document outlines the Nudge project's development plan and feature roadmap.

## Status Key

- ✅ **Done** — Implemented and released
- 🚧 **In Progress** — Under development
- 🎯 **Planned** — Requirements defined, not yet started

---

## Version History

### v0.3.0 — Auto Mode ✅

- Zsh auto mode (ghost text suggestions)
- Debounce delay configuration
- Tab to accept full suggestion
- Right Arrow to accept next word

### v0.4.0 — Error Diagnosis Enhancement ✅

**Goal**: Bring error diagnosis to the same level of project awareness as command completion.

**Completed**:
- ✅ Error diagnosis foundation (Zsh, PowerShell)
- ✅ stderr capture and analysis
- ✅ Tab to accept fix suggestion
- ✅ Unified diagnosis and completion context
- ✅ Full project context for diagnosis (Git, Node, Python, Rust, Docker)
- ✅ **Suggestion cache** (v0.4.2): LRU+TTL cache with stale-while-revalidate
  - Cache key: prefix + cwd + git_state + shell_mode
  - TTL: auto=5min, manual=10min
  - Context changes auto-invalidate
- ✅ **Bash popup selector** (v0.4.5): Multi-candidate browsing with fzf/sk/peco/builtin backends
  - LLM-generated multi-candidate suggestions
  - Risk preview and confirmation for dangerous commands
  - Configurable via environment variables

**Improvements**:
- Diagnosis now includes system info (OS, architecture, shell type)
- Diagnosis now includes full command history (session-based)
- Diagnosis now includes all project plugin context
- Unified `GatherParams` abstraction for context collection reuse

---

## Future Versions

### v0.5.0 — Smart History Analysis 🎯

- Frequency statistics and alias recommendations
- Common command sequence learning
- Error pattern recognition and prevention

### v0.6.0 — Extended Plugins 🎯

| Plugin | Trigger | Context Provided |
|---|---|---|
| **Kubernetes** | `kubectl/helm` | Current context, pods, config files |
| **Terraform** | `terraform` or `*.tf` | Resource definitions, workspace, state |
| **Database** | `psql/mysql/mongo` | Version, database list, connection config |

### v0.7.0 — Community Ecosystem 🎯

- WASM plugin system
- Custom prompt templates
- Plugin marketplace

### v1.0.0 — Stable Release 🎯

- Complete feature set
- Production-ready
- Comprehensive documentation and test coverage

---

## Completed Features

### Core

| Feature | Version | Description |
|---|---|---|
| AI command completion | v0.1.0 | LLM-powered command suggestions |
| Multi-shell support | v0.1.0 | Bash, Zsh, PowerShell, CMD |
| Privacy protection | v0.1.0 | Automatic sensitive data sanitization |
| Safety warnings | v0.1.0 | Dangerous command detection |
| Git plugin | v0.2.0 | Branch, commit, status context |
| Auto mode | v0.3.0 | Zsh ghost text suggestions |
| Error diagnosis | v0.3.0 | Failed command analysis and fix suggestions |
| Docker plugin | v0.3.0 | Container, image, compose context |
| Node.js plugin | v0.3.0 | package.json, scripts, dependencies |
| Python plugin | v0.3.0 | pyproject.toml, dependencies, virtualenv |
| Rust plugin | v0.3.0 | Cargo.toml, dependencies, workspace |
| Project-aware diagnosis | v0.4.0 | Diagnosis uses full project context |
| Suggestion cache | v0.4.2 | LRU+TTL cache to reduce LLM calls |
| Bash popup selector | v0.4.5 | Multi-candidate browsing with selector backends |

### Platform Support

| Shell | Manual Mode | Auto Mode | Error Diagnosis | Popup |
|---|---|---|---|---|
| Zsh | ✅ v0.1.0 | ✅ v0.3.0 | ✅ v0.3.0 | ❌ |
| Bash | ✅ v0.1.0 | ❌ | 🎯 Planned | ✅ v0.4.5 |
| PowerShell 7.2+ | ✅ v0.1.0 | ❌ | ✅ v0.3.0 | ❌ |
| PowerShell 5.1 | ✅ v0.1.0 | ❌ | ✅ v0.3.0 | ❌ |
| CMD | ✅ v0.1.0 | ❌ | ❌ | ❌ |

---

## Contributing

We welcome contributions to Nudge!

- Create or claim tasks in [Issues](https://github.com/Zhangtiande/nudge/issues)
- Follow development guidelines in `CLAUDE.md`
- Join [Discussions](https://github.com/Zhangtiande/nudge/discussions) for design proposals

---

*Last updated: 2026-02-07*
