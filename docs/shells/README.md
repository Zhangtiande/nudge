# Shell-Specific Guides

This directory separates Nudge behavior by shell, so feature scope and UX expectations are clear.

## Capability Matrix

| Shell | Runtime Mode(s) | Manual Fast Path | Multi-Candidate | `why/risk/diff` Display | Main UX Surface |
|---|---|---|---|---|---|
| Bash | `bash-inline`, `bash-popup` | `Ctrl+E` -> `bash-inline` (single candidate, fastest) | Yes (`bash-popup`) | Yes (popup rows + preview) | `Ctrl+E`, `Alt+/` |
| Zsh | `zsh-inline`, `zsh-auto` | `Ctrl+E` -> `zsh-inline` (single candidate, fastest) | No | Yes (overlay line / RPROMPT) | Ghost text + overlay |
| PowerShell | `ps-inline` | `Ctrl+E` -> `ps-inline` (single candidate, fastest) | No | No structured panel today | Manual completion + diagnosis |
| CMD | `cmd-inline` | `nudge-complete` macro -> `cmd-inline` | No | No | Manual completion |

## Documents

- [Bash Guide](./bash.md)
- [Zsh Guide](./zsh.md)
- [PowerShell Guide](./powershell.md)
- [CMD Guide](./cmd.md)

## Why This Split Exists

Nudge currently has shell-specific request modes and UI constraints. A single cross-shell document tends to mix incompatible assumptions (candidate count, overlay budget, keybindings, and explanation density).

## Baseline Rule

- Keep a manual single-candidate fast path in every shell integration.
- For Bash/Zsh/PowerShell this path is `Ctrl+E`.
- Advanced surfaces (popup/auto/overlay) are layered on top of that baseline.
