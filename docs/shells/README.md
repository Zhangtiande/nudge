# Shell-Specific Guides

This directory separates Nudge behavior by shell, so feature scope and UX expectations are clear.

## Capability Matrix

| Shell | Runtime Mode(s) | Multi-Candidate | `why/risk/diff` Display | Main UX Surface |
|---|---|---|---|---|
| Bash | `bash-popup` | Yes (popup list) | Yes (popup rows + preview) | `Ctrl+E`, `Alt+/` |
| Zsh | `zsh-inline`, `zsh-auto` | No (single candidate) | Yes (overlay line / RPROMPT) | Ghost text + overlay |
| PowerShell | `ps-inline` | No | No structured panel today | Manual completion + diagnosis |
| CMD | `cmd-inline` | No | No | Manual completion |

## Documents

- [Bash Guide](./bash.md)
- [Zsh Guide](./zsh.md)
- [Prompt and UX Improvement Plan](./prompt-and-ux-improvements.md)

## Why This Split Exists

Nudge currently has shell-specific request modes and UI constraints. A single cross-shell document tends to mix incompatible assumptions (candidate count, overlay budget, keybindings, and explanation density).
