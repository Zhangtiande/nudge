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
- Smooth buffer change detection via `zle-line-pre-redraw` hook
- Async background completion with `zle -F` file descriptor monitoring
- Tab accepts full suggestion
- Right Arrow accepts word-by-word

**Recommendation**: If you want auto mode, use Zsh. It's the default shell on macOS Catalina and later.

### Bash (Manual Mode Only)

**Auto mode is not supported in Bash.** This is a fundamental limitation of Bash readline, not a technical oversight:

**Why?** Bash readline lacks the async architecture needed for auto mode:
1. **No buffer change detection hook**: Bash readline doesn't trigger events when the user types. Zsh has `zle-line-pre-redraw` for this.
2. **No async completion**: Bash readline is fully synchronous. Zsh can run completions in the background with `zle -F` and update the display when done.
3. **No inline preview display**: Bash readline has no mechanism like Zsh's `POSTDISPLAY` to show ghost text without modifying the actual buffer.

These are architectural choices in Bash's readline library and cannot be worked around in a shell script.

**What you can do:**
- Use **manual mode** (`Ctrl+E`) in Bash - this works perfectly
- **Switch to Zsh** for auto mode - it's now the default shell on macOS
- Consider alternatives like `ble.sh` (though this is another tool to maintain and has its own limitations)

### PowerShell (Manual Mode Only)

PowerShell 7.2+ has a native predictor API through PSReadLine, but **it is not compatible with LLM-based completion**.

> ⚠️ **Why Auto Mode Doesn't Work**: PSReadLine's predictor API has a strict **~20ms timeout** ([Microsoft Docs](https://learn.microsoft.com/en-us/powershell/scripting/dev-cross-plat/create-cmdline-predictor)). LLM responses typically take 200ms+ (even with local models like Ollama), making auto mode unusable.

**Use Manual Mode Instead:**
- Press `Ctrl+E` to trigger completion on demand
- This bypasses the PSReadLine predictor timeout and works reliably with any LLM backend

The `NudgePredictor` module is still installed for potential future optimizations, but manual mode is the recommended approach.

### PowerShell 5.1 (Windows - Manual Mode Only)

PowerShell 5.1 does not support the predictor API, so only manual mode is available:

- Press `Ctrl+E` to trigger completion
- Auto mode is not supported

## Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `trigger.mode` | string | `"manual"` | `"manual"` or `"auto"` |
| `trigger.hotkey` | string | `"\C-e"` | Hotkey for manual mode (Ctrl+E) |
| `trigger.auto_delay_ms` | integer | `500` | Debounce delay in milliseconds |

## Key Bindings

### Manual Mode (All Platforms)
- `Ctrl+E`: Trigger completion

### Auto Mode (Unix - Bash/Zsh)
- `Tab`: Accept full suggestion
- `Right Arrow`: Accept next word (Zsh only)
- `Ctrl+E`: Force trigger completion (bypasses debounce)

### Windows (PowerShell/CMD - Manual Mode Only)
- `Ctrl+E`: Trigger completion

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

### Preview not displaying correctly (Zsh)

- Ensure your terminal supports ANSI escape codes
- Try a different terminal emulator
- Check that you're using Zsh (not Bash)

### Bash: Auto mode not supported

This is expected behavior. Bash readline doesn't support the async event hooks required for auto mode. Use manual mode (`Ctrl+E`) instead, or switch to Zsh for auto mode support.

### PowerShell auto mode not working

**This is expected behavior.** PSReadLine's predictor API has a ~20ms timeout, which is incompatible with LLM response times (200ms+).

**Solution: Use manual mode (`Ctrl+E`)** - this is the recommended approach for PowerShell.

If you want to verify the technical limitation:

1. Check your LLM response time:
   ```powershell
   Measure-Command { nudge complete --format json --buffer "git " --cursor 4 --cwd . --session test }
   ```
   Any response time over 20ms will not work with PSReadLine's predictor API.

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
