# Auto Mode Guide

Nudge supports two trigger modes:

| Mode | Description | Supported Shells |
|------|-------------|------------------|
| **Manual** (default) | Press `Ctrl+E` to trigger | All shells |
| **Auto** | Ghost text appears as you type | Zsh only |

## Enabling Auto Mode

Edit your config file:

```yaml
trigger:
  mode: auto
  auto_delay_ms: 500    # Debounce delay (ms)
  zsh_ghost_owner: auto # auto | nudge | autosuggestions
```

- `auto`: Prefer `zsh-autosuggestions` when available, otherwise use Nudge ghost text
- `nudge`: Force Nudge to render ghost text
- `autosuggestions`: Reserve ghost text for `zsh-autosuggestions`
  - Nudge still fetches slow suggestions via overlay; accept key falls back to `Ctrl+G` to avoid taking over `Tab`

Restart your shell:
```bash
source ~/.zshrc
```

## How It Works

1. **Debouncing** - Waits for `auto_delay_ms` of idle time before requesting completion
2. **Ghost Text** - Suggestion appears as gray text after cursor
3. **Accept** - Press `Tab` for full suggestion, `Right Arrow` for next word

```
$ git sta|tus                    # Gray suggestion
         ↑ cursor

$ git status|                    # After Tab
            ↑ cursor moved
```

## Key Bindings

| Key | Action | Mode |
|-----|--------|------|
| `Ctrl+E` | Trigger completion | Both |
| `Tab` | Accept full suggestion | Auto |
| `Right Arrow` | Accept next word | Auto (Zsh) |
| `Ctrl+G` | Accept Nudge overlay/diagnosis suggestion when ghost owner is `autosuggestions` | Zsh |

## Shell Support

### Zsh (Recommended)

Full auto mode support:
- Ghost text via `POSTDISPLAY`
- Async completion with `zle -F`
- Tab accepts full suggestion
- Right Arrow accepts word-by-word

### Bash

**Auto mode not supported.** Bash readline lacks:
- Buffer change detection hooks
- Async completion mechanism
- Inline preview display

Use manual mode (`Ctrl+E`) instead, or switch to Zsh.

### PowerShell

**Auto mode not supported.** PSReadLine's predictor API has a ~20ms timeout, incompatible with LLM response times (200ms+).

Use manual mode (`Ctrl+E`) instead.

## Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `trigger.mode` | string | `manual` | `manual` or `auto` |
| `trigger.hotkey` | string | `\C-e` | Manual trigger key |
| `trigger.auto_delay_ms` | integer | 500 | Debounce delay |
| `trigger.zsh_ghost_owner` | string | `auto` | `auto`, `nudge`, or `autosuggestions` |

## Troubleshooting

**Suggestions not appearing**
```bash
nudge status          # Check daemon
nudge info            # Check config
```

**High latency**
- Reduce `auto_delay_ms` (e.g., 300)
- Use local LLM (Ollama)

**Tab not working**
- In auto mode, Tab accepts suggestions
- Use `Ctrl+E` for manual trigger

## Disabling Auto Mode

```yaml
trigger:
  mode: manual
```

Or temporarily:
```bash
export NUDGE_TRIGGER_MODE=manual
```

## See Also

- [Configuration Reference](configuration.md)
- [CLI Reference](cli-reference.md)
