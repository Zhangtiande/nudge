# Shell Guides

This section describes behavior per shell mode and integration constraints.

## Capability Matrix

| Shell | Modes | Fast Path | Multi-candidate | Auto Ghost Text | Diagnosis |
|---|---|---|---|---|---|
| Bash | `bash-inline`, `bash-popup` | `Ctrl+E` (`bash-inline`) | Yes (`bash-popup`) | No | Planned |
| Zsh | `zsh-inline`, `zsh-auto` | `Ctrl+E` (`zsh-inline`) | No popup selector | Yes | Yes |
| PowerShell | `ps-inline` | `Ctrl+E` | No | No | Yes |
| CMD | `cmd-inline` | `Ctrl+E` | No | No | No |

## Baseline Rule

`Ctrl+E` manual path is always kept as the fastest single-candidate baseline path across shells.

## Per-Shell Docs

- [Bash](bash.md)
- [Zsh](zsh.md)
- [PowerShell](powershell.md)
- [CMD](cmd.md)
