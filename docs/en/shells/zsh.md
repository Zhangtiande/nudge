# Zsh Guide

[English](zsh.md) | [中文](../../zh/shells/zsh.md)

Zsh is the most complete integration path — manual, auto ghost text, overlay, and error diagnosis.

## Modes

| Mode | Trigger | Description |
|---|---|---|
| `zsh-inline` | `Ctrl+E` | Fast single-candidate apply (always available) |
| `zsh-auto` | Typing | Live ghost-text or overlay suggestions |

## Quick Start

Manual mode (works out of the box):

```bash
# Type a partial command, then press Ctrl+E
git st<Ctrl+E>
# → git status
```

Enable auto mode:

```yaml
# ~/.nudge/config/config.yaml
trigger:
  mode: auto
```

```bash
nudge restart
```

## Ghost Text Ownership

The `trigger.zsh_ghost_owner` setting controls who renders the gray suggestion text.

| Value | Ghost text by | Nudge display | Accept key |
|---|---|---|---|
| `auto` | autosuggestions (if present), else Nudge | Overlay or ghost | `Tab` or `Ctrl+G` |
| `nudge` | Nudge | Ghost text | `Tab` |
| `autosuggestions` | zsh-autosuggestions | Overlay | `Ctrl+G` |

### `auto` (default)

Detects `zsh-autosuggestions` at load time:
- Present → Nudge uses overlay; ghost text stays with autosuggestions
- Absent → Nudge takes ghost text ownership

### `nudge`

Force Nudge to own ghost text. Overrides autosuggestions.

```yaml
trigger:
  zsh_ghost_owner: nudge
```

### `autosuggestions`

Reserve ghost text for autosuggestions. Nudge always uses overlay.

```yaml
trigger:
  zsh_ghost_owner: autosuggestions
```

## Overlay Backend

When Nudge uses overlay mode (not owning ghost text), two rendering backends are available:

| Backend | Method | Behavior |
|---|---|---|
| `message` (default) | `zle -M` | Message line below prompt; clears on next keystroke |
| `rprompt` | `RPS1` | Right prompt; persists until next suggestion |

```yaml
trigger:
  zsh_overlay_backend: rprompt  # or message
```

## Key Bindings

| Key | Action | Condition |
|---|---|---|
| `Ctrl+E` | Manual completion (immediate LLM request) | Always |
| `Tab` | Accept full ghost suggestion | Nudge owns ghost text |
| `Right Arrow` | Accept next word | Nudge owns ghost text |
| `Ctrl+G` | Accept overlay suggestion | Overlay mode |
| `F1` | Toggle explanation detail (why/diff/risk) | Always |

## Error Diagnosis

When `diagnosis.enabled: true`, Zsh integration automatically analyzes failed commands.

### How it works

1. Before command execution (`preexec`): stderr is redirected to a temporary file
2. After command execution (`precmd`): if exit code ≠ 0, Nudge sends the command + captured stderr to the LLM
3. The diagnosis message appears below the prompt
4. A suggested fix is shown as inline text — press `Tab` to accept

### Configuration

```yaml
diagnosis:
  enabled: true
  capture_stderr: true
  auto_suggest: true
  max_stderr_size: 4096
```

### Interactive command exclusions

Stderr capture breaks interactive programs (vim, ssh, top). These are excluded by default via `diagnosis.interactive_commands`. Add custom exclusions in your config:

```yaml
diagnosis:
  interactive_commands:
    - vim
    - nvim
    - ssh
    - your-custom-tool
```

Check the current list:

```bash
nudge info --field interactive_commands
```

## Health Check: `nudge doctor zsh`

```bash
nudge doctor zsh
```

Checks performed:

| Check | What it verifies |
|---|---|
| Integration sourced | `integration.zsh` is loaded |
| Key bindings | `Ctrl+E`, `Tab`, `F1` are registered |
| Hooks | `precmd`, `preexec` hooks are installed |
| Daemon | Daemon is reachable via IPC |
| Ghost owner | Config matches runtime state |
| Overlay backend | Backend is functional |

Each check prints `OK` or `WARN`. To fix warnings:

```bash
nudge setup zsh --force
nudge restart
```

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| No suggestions at all | Daemon not running | `nudge start` |
| `Ctrl+E` does nothing | Integration not sourced | `nudge setup zsh --force && source ~/.zshrc` |
| Ghost text flickers | Terminal redraw conflict | Try `zsh_overlay_backend: rprompt` |
| Suggestions overwrite autosuggestions | Ghost owner conflict | Set `zsh_ghost_owner: autosuggestions` |
| `F1` does nothing | Terminal maps F1 to help | Check terminal key settings |
| Diagnosis doesn't appear | `diagnosis.enabled` is false | Set to `true` and restart |
| stderr lost for interactive tools | Missing from exclusion list | Add to `interactive_commands` |
| Stale suggestions after cd | Cache not invalidated | Should auto-invalidate; check `RUST_LOG=debug` |

## Boundaries

- Overlay density depends on terminal width
- Function key mapping (`F1`) may vary across terminal emulators
- Stderr capture is Zsh-specific; it does not work in other shells
- Ghost text requires ANSI color support in the terminal
