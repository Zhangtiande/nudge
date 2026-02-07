# Auto Mode Guide

Auto mode provides live ghost-text suggestions while you type.

## Scope

- Supported shell: **Zsh only**
- Bash/PowerShell/CMD stay on manual trigger mode

## Quick Enable

`~/.nudge/config/config.yaml`:

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

## Recommended Zsh Paths

### A) Nudge owns ghost text

```yaml
trigger:
  mode: auto
  zsh_ghost_owner: nudge
```

### B) zsh-autosuggestions owns ghost text (overlay mode)

```yaml
trigger:
  mode: auto
  zsh_ghost_owner: autosuggestions
```

In overlay mode, use `Ctrl+G` to accept Nudge suggestion.

## Key Bindings (Zsh)

- `Tab`: accept suggestion (when Nudge owns ghost text)
- `Right Arrow`: accept next word
- `F1`: toggle explanation detail
- `Ctrl+G`: accept overlay suggestion when autosuggestions owns ghost text

## Troubleshooting

```bash
nudge doctor zsh
nudge info --field zsh_ghost_owner
nudge info --field zsh_overlay_backend
```

If behavior looks stale:

```bash
nudge setup zsh --force
nudge restart
```

## Boundaries

- No cross-shell auto mode fallback
- Some terminals may map function keys differently; verify with `doctor`
