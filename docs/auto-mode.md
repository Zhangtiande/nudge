# Auto Mode Documentation

Nudge supports two trigger modes for command completion:

1. **Manual Mode** (default): Press `Ctrl+E` to trigger completion
2. **Auto Mode**: Suggestions appear automatically as you type

## Enabling Auto Mode

Edit your configuration file (`~/.config/nudge/config.yaml` on Linux or `~/Library/Application Support/nudge/config.yaml` on macOS):

```yaml
trigger:
  mode: auto              # Change from "manual" to "auto"
  hotkey: "\C-e"          # Manual mode hotkey (still works in auto mode)
  auto_delay_ms: 500      # Delay before triggering completion (milliseconds)
```

After changing the configuration, restart your shell or run:

```bash
source ~/.bashrc   # For Bash
source ~/.zshrc    # For Zsh
```

## How Auto Mode Works

1. **Debouncing**: When you type, Nudge waits for `auto_delay_ms` (default 500ms) of idle time before requesting a completion
2. **Inline Preview**: The suggestion appears as gray text after your cursor
3. **Accept**: Press `Tab` to accept the full suggestion, or `Right Arrow` to accept word-by-word

### Visual Example

```
$ git sta|tus                    # Gray text shows the suggestion
         ↑ cursor here

$ git status|                    # After pressing Tab
            ↑ cursor moved to end
```

## Shell Support

### Zsh (Recommended)

Zsh has excellent support for auto mode through its ZLE (Zsh Line Editor):

- Full inline preview with `POSTDISPLAY`
- Smooth buffer change detection via `zle-line-pre-redraw`
- Tab accepts full suggestion
- Right Arrow accepts word-by-word

### Bash (Limited)

Bash's readline has limited support for inline preview:

- Preview display uses ANSI escape codes (may not work in all terminals)
- Buffer change detection is less reliable
- For better auto mode support in Bash, consider using [ble.sh](https://github.com/akinomyoga/ble.sh)

## Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `trigger.mode` | string | `"manual"` | `"manual"` or `"auto"` |
| `trigger.hotkey` | string | `"\C-e"` | Hotkey for manual mode (Ctrl+E) |
| `trigger.auto_delay_ms` | integer | `500` | Debounce delay in milliseconds |

## Key Bindings

### Manual Mode
- `Ctrl+E`: Trigger completion

### Auto Mode
- `Tab`: Accept full suggestion
- `Right Arrow`: Accept next word (Zsh only)
- `Ctrl+E`: Force trigger completion (bypasses debounce)

## Troubleshooting

### Suggestions not appearing

1. Check if auto mode is enabled:
   ```bash
   nudge info --field trigger_mode
   ```

2. Verify the daemon is running:
   ```bash
   nudge status
   ```

3. Check the delay setting:
   ```bash
   nudge info --field auto_delay_ms
   ```

### Preview not displaying correctly

- Ensure your terminal supports ANSI escape codes
- Try a different terminal emulator
- In Bash, consider switching to Zsh for better support

### High latency

1. Reduce the debounce delay:
   ```yaml
   trigger:
     auto_delay_ms: 300  # Faster triggering
   ```

2. Check your LLM endpoint latency:
   ```bash
   time nudge complete --buffer "git sta" --cursor 7 --cwd . --session test
   ```

### Tab key not working

In auto mode, Tab is bound to accept suggestions. If you need traditional Tab completion:

1. Use `Ctrl+E` for Nudge completion
2. Or switch back to manual mode

## Performance Considerations

Auto mode makes more API calls than manual mode. To optimize:

1. **Increase debounce delay**: Higher `auto_delay_ms` means fewer API calls
2. **Use local LLM**: Ollama or other local models have lower latency
3. **Enable caching**: Nudge caches recent completions automatically

## Disabling Auto Mode

To switch back to manual mode:

```yaml
trigger:
  mode: manual
```

Or temporarily disable by setting the environment variable:

```bash
export NUDGE_TRIGGER_MODE=manual
```
