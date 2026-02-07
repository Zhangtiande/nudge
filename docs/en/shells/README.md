# Shell Guides

[English](README.md) | [中文](../../zh/shells/README.md)

Per-shell behavior, integration modes, and feature availability.

## Capability Matrix

| Shell | Modes | Fast Path | Popup | Auto Ghost | Diagnosis | Cache | Plugins |
|---|---|---|---|---|---|---|---|
| **Zsh** | `zsh-inline`, `zsh-auto` | `Ctrl+E` | No | Yes | Yes | Yes | All |
| **Bash** | `bash-inline`, `bash-popup` | `Ctrl+E` | `Alt+/` (fzf/sk/peco/builtin) | No | Planned | Yes | All |
| **PowerShell** | `ps-inline` | `Ctrl+E` | No | No | Yes | Yes | All |
| **CMD** | `cmd-inline` | `Ctrl+E` | No | No | No | Yes | All |

**One-line positioning**:

- **Zsh**: Full-feature flagship — auto ghost text, overlay, diagnosis, every key binding
- **Bash**: Best multi-candidate experience — popup selector with risk preview
- **PowerShell**: Windows-native path — PSReadLine integration with error diagnosis
- **CMD**: Minimal viable — doskey macros for basic completion

## Baseline Rule

`Ctrl+E` manual path is always the fastest single-candidate baseline across all shells.

## Per-Shell Docs

- [Zsh](zsh.md) · [中文](../../zh/shells/zsh.md) — Auto mode, ghost ownership, overlay, diagnosis
- [Bash](bash.md) · [中文](../../zh/shells/bash.md) — Popup selector, multi-candidate workflow
- [PowerShell](powershell.md) · [中文](../../zh/shells/powershell.md) — PSReadLine predictor, diagnosis
- [CMD](cmd.md) · [中文](../../zh/shells/cmd.md) — Doskey macros, basic setup
