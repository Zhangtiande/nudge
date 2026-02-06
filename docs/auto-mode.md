# Auto Mode Guide

Nudge supports two trigger modes:

| Mode | Description | Supported Shells |
|------|-------------|------------------|
| **Manual** (default) | Press `Ctrl+E` to trigger | All shells |
| **Auto** | Ghost/overlay suggestions appear while typing | Zsh only |

## Enabling Auto Mode

Edit your config file:

```yaml
trigger:
  mode: auto
  auto_delay_ms: 500         # Delay hint for non-Zsh integrations (Zsh is event-driven)
  zsh_ghost_owner: auto      # auto | nudge | autosuggestions
  zsh_overlay_backend: message # message | rprompt
```

- `zsh_ghost_owner: auto`
  - Prefer `zsh-autosuggestions` when present
  - Fallback to Nudge ghost text if not present
- `zsh_ghost_owner: nudge`
  - Nudge owns ghost text + partial accept keys
- `zsh_ghost_owner: autosuggestions`
  - Keep `Tab` for autosuggestions
  - Nudge uses overlay accept on `Ctrl+G`
- `zsh_overlay_backend: message`
  - Render overlay via `zle -M` message line (default)
- `zsh_overlay_backend: rprompt`
  - Render overlay via right prompt (`RPS1`) with save/restore semantics

Restart your shell:

```bash
source ~/.zshrc
```

## How It Works

1. **Event-driven fetch**
   - Nudge fetches when input events settle (`PENDING`/`KEYS_QUEUED_COUNT` drained)
   - No sleep-based debounce loop in Zsh integration
2. **Daemon cache fast path**
   - Auto requests are served by daemon cache when hot
   - Slow refresh can update the preview/overlay asynchronously
3. **Dual presentation**
   - Ghost text (`POSTDISPLAY`) when Nudge owns ghost rendering
   - Overlay line when `autosuggestions` owns ghost rendering
   - `message` backend uses plain `[nudge] [risk]` badges for max compatibility
   - `rprompt` backend uses colored badges
4. **Progressive accept**
   - Full / word acceptance (`Tab` and `Right Arrow`)

`risk` in overlay is driven by daemon safety results (`NUDGE_WARNING`), not local shell-side regex detection.

## Key Bindings

| Key | Action | Mode |
|-----|--------|------|
| `Ctrl+E` | Trigger completion | Both |
| `Tab` | Accept full suggestion | Auto (Nudge ghost owner) |
| `Right Arrow` | Accept next word | Auto (Zsh) |
| `F1` | Toggle explanation details (`why/risk/diff`) | Auto (Zsh) |
| `Ctrl+G` | Accept Nudge overlay/diagnosis suggestion when ghost owner is `autosuggestions` | Zsh |

## Shell Support

### Zsh (Recommended)

Full auto mode support:
- Ghost text via `POSTDISPLAY` (when Nudge owns ghost)
- Overlay mode via hooks (`line-pre-redraw`, `line-finish`)
- Async completion via `zle -F`
- Explanation layer with `F1`
- Partial accept control by word (`Right Arrow`)

### Bash

**Auto mode not supported.** Bash readline lacks:
- Reliable buffer-change hooks
- ZLE-style async redraw hooks
- Native ghost/overlay rendering primitives

Use manual mode (`Ctrl+E`) instead.

### PowerShell

**Auto mode not supported.** PSReadLine predictor deadlines are too strict for LLM latency.

Use manual mode (`Ctrl+E`) instead.

## Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `trigger.mode` | string | `manual` | `manual` or `auto` |
| `trigger.hotkey` | string | `\C-e` | Manual trigger key |
| `trigger.auto_delay_ms` | integer | 500 | Delay hint (legacy for Zsh auto mode) |
| `trigger.zsh_ghost_owner` | string | `auto` | `auto`, `nudge`, or `autosuggestions` |
| `trigger.zsh_overlay_backend` | string | `message` | `message` or `rprompt` |

## Troubleshooting

**Suggestions not appearing**

```bash
nudge status
nudge info
nudge doctor zsh
```

**Overlay conflicts with other plugins**
- Set `trigger.zsh_ghost_owner: autosuggestions`
- Keep Nudge acceptance on `Ctrl+G`
- Check `nudge doctor zsh` key-binding output

**Prompt redraw feels noisy**
- Use `trigger.zsh_overlay_backend: message` (default)
- `rprompt` backend redraws right prompt by design

## Disabling Auto Mode

```yaml
trigger:
  mode: manual
```

## See Also

- [Configuration Reference](configuration.md)
- [CLI Reference](cli-reference.md)
