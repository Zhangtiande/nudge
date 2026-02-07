# Auto Mode Guide

[English](auto-mode.md) | [中文](../zh/auto-mode.md)

Auto mode provides live ghost-text suggestions while you type — no hotkey needed.

## Scope

- Supported shell: **Zsh only** (full ghost-text and overlay)
- PowerShell 7.2+: Experimental auto mode via PSReadLine predictor (see [PowerShell guide](shells/powershell.md))
- Bash/CMD: Manual trigger only (`Ctrl+E`)

## Quick Enable

Edit `~/.nudge/config/config.yaml`:

```yaml
trigger:
  mode: auto
  zsh_ghost_owner: auto
  zsh_overlay_backend: message
```

Apply:

```bash
nudge restart
```

## Ghost Text Ownership (`zsh_ghost_owner`)

This setting determines who renders the gray suggestion text after your cursor.

### `auto` (default)

Nudge detects whether `zsh-autosuggestions` is loaded:

- **If autosuggestions is present**: Nudge defers ghost text to autosuggestions and renders its own suggestion in an overlay line (message or rprompt). Accept Nudge's suggestion with `Ctrl+G`.
- **If autosuggestions is absent**: Nudge takes ownership of ghost text. Accept with `Tab`.

This is the safest option for most users.

### `nudge`

Force Nudge to own ghost text regardless of other plugins. This replaces any `zsh-autosuggestions` ghost text with Nudge's LLM suggestion.

```yaml
trigger:
  mode: auto
  zsh_ghost_owner: nudge
```

Best for users who prefer LLM suggestions over history-based autosuggestions.

### `autosuggestions`

Reserve ghost text for `zsh-autosuggestions`. Nudge always uses the overlay display.

```yaml
trigger:
  mode: auto
  zsh_ghost_owner: autosuggestions
```

Best for users who rely on autosuggestions for fast history recall and want Nudge suggestions shown separately.

## Overlay Backend (`zsh_overlay_backend`)

When Nudge does **not** own ghost text (i.e., `autosuggestions` or `auto` with autosuggestions present), it displays suggestions via an overlay. Two backends are available:

### `message` (default)

Uses `zle -M` to render a message line below the prompt.

- Less prompt redraw flickering
- Disappears on next keystroke
- Works in all terminals

### `rprompt`

Renders the suggestion in the right prompt (`RPS1`).

- Persistent until new suggestion arrives
- May conflict with existing right prompt customizations
- Better visibility for short suggestions

```yaml
trigger:
  zsh_overlay_backend: rprompt
```

## Event-Driven Fetch Mechanism

Auto mode does not poll. Instead, it works as follows:

1. You type a character → Zsh widget fires
2. Integration checks `$PENDING` and `$KEYS_QUEUED_COUNT` — if more keys are queued, it skips (debounce)
3. After `auto_delay_ms` (default 500ms), a background fetch is triggered
4. When the response arrives, a generation counter prevents stale responses from overwriting newer ones
5. The suggestion is rendered (ghost text or overlay, depending on ownership)

This design avoids unnecessary LLM calls during fast typing while keeping suggestions fresh.

### Tuning `auto_delay_ms`

| Value | Behavior |
|---|---|
| `200` | Very responsive, more LLM calls |
| `500` | Balanced (default) |
| `1000` | Conservative, fewer calls |

```yaml
trigger:
  auto_delay_ms: 300
```

## Key Bindings (Zsh Auto Mode)

| Key | Action | When |
|---|---|---|
| `Tab` | Accept full suggestion | Nudge owns ghost text |
| `Right Arrow` | Accept next word | Nudge owns ghost text |
| `Ctrl+G` | Accept Nudge suggestion | Overlay mode (autosuggestions owns ghost) |
| `F1` | Toggle explanation detail | Always (shows `why`, `diff`, `risk`) |
| `Ctrl+E` | Manual completion | Always (bypasses auto, immediate request) |

## Suggestion Cache Integration

Auto mode benefits significantly from the suggestion cache:

- Repeated prefixes return cached results instantly (no LLM call)
- Context changes (cd, git commit) invalidate relevant entries automatically
- Stale-while-revalidate returns old results immediately while refreshing in background
- Auto mode uses a 5-minute TTL (configurable via `cache.ttl_auto_ms`)

## Troubleshooting

Check current settings:

```bash
nudge doctor zsh
nudge info --field zsh_ghost_owner
nudge info --field zsh_overlay_backend
nudge info --field trigger_mode
```

If behavior looks stale or broken:

```bash
nudge setup zsh --force
nudge restart
```

**Common issues**:

| Symptom | Likely cause | Fix |
|---|---|---|
| No suggestions appear | Daemon not running | `nudge start` |
| Ghost text flickers | Terminal redraw conflict | Try `zsh_overlay_backend: rprompt` |
| Suggestions overwrite autosuggestions | Ghost owner conflict | Set `zsh_ghost_owner: autosuggestions` |
| Suggestions arrive too late | High `auto_delay_ms` | Lower to 300ms |
| `Ctrl+G` does nothing | Not in overlay mode | Check `nudge info --field zsh_ghost_owner` |

## Boundaries

- No cross-shell auto mode fallback — Bash/CMD stay on manual trigger
- Some terminals may map function keys differently; verify with `nudge doctor zsh`
- Auto mode requires the daemon to be running; it does not start it automatically
- Ghost text rendering depends on terminal ANSI support
